"""Scaffold and update reproducible project environments with Nix."""

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

VERSION = "0.4.0"
NIXPKGS_INPUT = "github:NixOS/nixpkgs/nixpkgs-unstable"
PLAINIX_INPUT = "github:yawarakatai/plainix"
PACKAGE_SEARCH_URL = "https://search.nixos.org/packages?channel=unstable"
SUPPORTED_SYSTEMS = ("x86_64-linux", "aarch64-linux", "aarch64-darwin")
PROJECT_FILES = ("plainix.nix", "flake.nix", "flake.lock")
NO_TOOLS = "none"
PRESETS = json.loads(Path(__file__).with_name("presets.json").read_text("utf-8"))


class PlainixError(Exception):
    pass


def current_system(machine, operating_system):
    systems = {
        ("x86_64", "Linux"): "x86_64-linux",
        ("aarch64", "Linux"): "aarch64-linux",
        ("arm64", "Darwin"): "aarch64-darwin",
    }
    try:
        return systems[(machine, operating_system)]
    except KeyError:
        raise PlainixError(
            f"Plainix: unsupported platform `{machine}-{operating_system}`"
        ) from None


def available_for(system):
    return [preset for preset in PRESETS if system in preset["systems"]]


def resolve(ids, system):
    selected = []
    for tool in ids:
        preset = next((item for item in PRESETS if item["id"] == tool), None)
        if preset is None:
            available = ", ".join(item["id"] for item in available_for(system))
            raise PlainixError(
                f"Plainix: unknown tool `{tool}`. Available tools: {available}"
            )
        if system not in preset["systems"]:
            raise PlainixError(f"Plainix: tool `{tool}` is not available on `{system}`")
        if preset not in selected:
            selected.append(preset)
    return selected


def packages_for(presets):
    packages = []
    for preset in presets:
        for package in preset["packages"]:
            if package not in packages:
                packages.append(package)
    return packages


def shared_systems(presets):
    return [
        system
        for system in SUPPORTED_SYSTEMS
        if all(system in preset["systems"] for preset in presets)
    ]


def selection_menu(system):
    entries = [f"{NO_TOOLS}\t(none) — start with an empty environment"]
    entries += [
        f"{preset['id']}\t{preset['name']} — {preset['description']}"
        for preset in available_for(system)
    ]
    return "\n".join(entries) + "\n"


def parse_selection(output):
    ids = [line.split("\t", 1)[0] for line in output.splitlines() if line]
    return [tool for tool in ids if tool != NO_TOOLS]


def select_interactively(system):
    if not (sys.stdin.isatty() and sys.stderr.isatty()):
        raise PlainixError(
            "Plainix: interactive selection requires a terminal; "
            "use `plainix init --with <tool>` or `plainix init --empty`"
        )
    try:
        completed = subprocess.run(
            [
                "fzf",
                "--multi",
                "--layout=reverse",
                "--delimiter=\t",
                "--with-nth=2..",
                "--bind=space:toggle+down",
                "--prompt=Select tools> ",
                "--header=Space to select, Enter to continue, type to filter",
            ],
            check=False,
            input=selection_menu(system),
            stdout=subprocess.PIPE,
            text=True,
        )
    except OSError as error:
        raise PlainixError(f"Plainix: could not run fzf: {error}") from error
    if completed.returncode in (1, 130):
        raise PlainixError("Plainix: initialization cancelled")
    if completed.returncode != 0:
        raise PlainixError(f"Plainix: fzf exited with {completed.returncode}")
    return parse_selection(completed.stdout)


def check_directory(directory):
    conflicts = [
        directory / file for file in PROJECT_FILES if (directory / file).exists()
    ]
    if conflicts:
        paths = ", ".join(f"`{path}`" for path in conflicts)
        raise PlainixError(
            f"Plainix: refusing to overwrite {paths}; "
            "remove or move the existing files first"
        )


def write_atomic(path, contents):
    temporary = path.with_name(f".{path.name}.plainix-{os.getpid()}.tmp")
    try:
        temporary.write_bytes(contents)
    except OSError as error:
        temporary.unlink(missing_ok=True)
        raise PlainixError(
            f"Plainix: could not write temporary file `{temporary}`: {error}"
        ) from error
    try:
        os.replace(temporary, path)
    except OSError as error:
        temporary.unlink(missing_ok=True)
        raise PlainixError(f"Plainix: could not replace `{path}`: {error}") from error


def render_list(values):
    if not values:
        return "[ ]"
    lines = "\n".join(f'    "{value}"' for value in values)
    return f"[\n{lines}\n  ]"


def render_plainix(systems, presets, packages):
    tools = ""
    if presets:
        names = ", ".join(preset["name"] for preset in presets)
        tools = f"# Initial tools: {names}\n"
    return (
        f"{tools}# Package source: {NIXPKGS_INPUT} (pinned in flake.lock)\n"
        f"# Find package names: {PACKAGE_SEARCH_URL}\n\n"
        "{\n"
        f"  systems = {render_list(systems)};\n\n"
        f"  packages = {render_list(packages)};\n"
        "}\n"
    )


def render_flake():
    return (
        "{\n"
        '  description = "Project environment generated by Plainix";\n\n'
        f'  inputs.nixpkgs.url = "{NIXPKGS_INPUT}";\n'
        "  inputs.plainix = {\n"
        f'    url = "{PLAINIX_INPUT}";\n'
        '    inputs.nixpkgs.follows = "nixpkgs";\n'
        "  };\n\n"
        "  outputs = inputs:\n"
        "    inputs.plainix.lib.project inputs ./plainix.nix;\n"
        "}\n"
    )


def is_direnv_ignored(line):
    return line.strip() in (".direnv", ".direnv/", "/.direnv", "/.direnv/")


def ensure_gitignore(path):
    try:
        existing = path.read_bytes()
    except FileNotFoundError:
        write_atomic(path, b".direnv/\n")
        return "created"
    except OSError as error:
        raise PlainixError(f"Plainix: could not read `{path}`: {error}") from error

    try:
        text = existing.decode("utf-8")
    except UnicodeDecodeError as error:
        raise PlainixError(
            f"Plainix: could not update `{path}` because it is not UTF-8: {error}"
        ) from error
    if any(is_direnv_ignored(line) for line in text.splitlines()):
        return "present"

    if existing and not existing.endswith(b"\n"):
        existing += b"\n"
    write_atomic(path, existing + b".direnv/\n")
    return "updated"


def is_git_repository(directory):
    try:
        completed = subprocess.run(
            ["git", "-C", str(directory), "rev-parse", "--is-inside-work-tree"],
            check=False,
            capture_output=True,
        )
    except OSError:
        return False
    return completed.returncode == 0


def prepare_git_files(directory, files):
    """Mark files intent-to-add so Nix can see them; return an error message or None."""
    try:
        completed = subprocess.run(
            [
                "git",
                "-C",
                str(directory),
                "add",
                "--intent-to-add",
                "--force",
                "--",
                *files,
            ],
            check=False,
            capture_output=True,
            text=True,
            errors="replace",
        )
    except OSError as error:
        return str(error)
    if completed.returncode == 0:
        return None
    return completed.stderr.strip() or f"git add exited with {completed.returncode}"


def lock_flake(directory):
    # Locking only needs flake.nix. Running it in the project would copy the
    # whole directory, including any large assets, into the Nix store.
    with tempfile.TemporaryDirectory(prefix="plainix-lock-") as name:
        workspace = Path(name)
        try:
            shutil.copy(directory / "flake.nix", workspace / "flake.nix")
        except OSError as error:
            raise PlainixError(
                f"Plainix: could not prepare locking: {error}"
            ) from error

        print(
            "Creating flake.lock with Nix (this may take a moment)...", file=sys.stderr
        )
        try:
            completed = subprocess.run(
                ["nix", "flake", "lock", "path:."], check=False, cwd=workspace
            )
        except OSError as error:
            raise PlainixError(
                f"Plainix: could not run `nix flake lock`: {error}\n"
                "Install Nix and enable `nix-command` and `flakes`, then try again."
            ) from error
        if completed.returncode != 0:
            raise PlainixError(
                f"Plainix: `nix flake lock` failed with exit code {completed.returncode}. "
                "See the Nix output above.\n"
                "Ensure Nix has `experimental-features = nix-command flakes` enabled."
            )
        try:
            lock = (workspace / "flake.lock").read_bytes()
        except OSError as error:
            raise PlainixError(
                f"Plainix: `nix flake lock` did not produce a readable `flake.lock`: {error}"
            ) from error
    write_atomic(directory / "flake.lock", lock)


def remove_generated(paths):
    problems = []
    for path in reversed(paths):
        try:
            path.unlink(missing_ok=True)
        except OSError as error:
            problems.append(f"could not remove `{path}`: {error}")
    return problems


@dataclass
class InitResult:
    packages: list
    systems: list
    envrc_created: bool
    git_repository: bool
    git_files: list
    git_error: str | None = None
    gitignore: str | None = None
    gitignore_error: str | None = None


def initialize(directory, presets, git_repository, lock):
    check_directory(directory)

    packages = packages_for(presets)
    systems = shared_systems(presets)
    envrc_path = directory / ".envrc"
    envrc_created = not envrc_path.exists()
    git_files = list(PROJECT_FILES)
    if envrc_created:
        git_files.append(".envrc")
    generated = [directory / file for file in git_files]

    try:
        write_atomic(
            directory / "plainix.nix",
            render_plainix(systems, presets, packages).encode("utf-8"),
        )
        write_atomic(directory / "flake.nix", render_flake().encode("utf-8"))
        if envrc_created:
            write_atomic(envrc_path, b"# Generated by Plainix\nuse flake\n")
        lock(directory)
    except BaseException as error:
        problems = remove_generated(generated)
        if problems:
            raise PlainixError(
                f"{error}\nPlainix: cleanup was incomplete: {'; '.join(problems)}"
            ) from error
        raise

    result = InitResult(packages, systems, envrc_created, git_repository, git_files)
    if git_repository:
        try:
            result.gitignore = ensure_gitignore(directory / ".gitignore")
            if result.gitignore in ("created", "updated"):
                git_files.append(".gitignore")
        except PlainixError as error:
            result.gitignore = "failed"
            result.gitignore_error = str(error)
        result.git_error = prepare_git_files(directory, git_files)
    return result


def print_init_result(presets, result):
    tools = ", ".join(preset["name"] for preset in presets) or "none"
    print(f"Tools: {tools}")
    print(f"Platforms: {', '.join(result.systems)}")
    print(f"Packages: {', '.join(result.packages) or 'none'}")
    print()
    for file in PROJECT_FILES:
        print(f"Created {file}")

    if result.envrc_created:
        print("Created .envrc")
    else:
        print("Kept existing .envrc; add `use flake` manually if needed")

    if result.gitignore == "created":
        print("Created .gitignore with .direnv/")
    elif result.gitignore == "updated":
        print("Added .direnv/ to .gitignore")
    elif result.gitignore == "failed":
        print(f"Could not add .direnv/ to .gitignore: {result.gitignore_error}")

    if result.git_repository:
        files = " ".join(result.git_files)
        print()
        if result.git_error is None:
            print(
                "Prepared generated files for Git so Nix can see them; "
                "contents remain unstaged."
            )
            print("To commit them later:")
        else:
            print(f"Could not prepare generated files for Git: {result.git_error}")
            print("Before running nix develop, stage them manually:")
        print(f"  git add {files}")

    if result.envrc_created:
        print()
        print("If you use direnv, review the files and run:")
        print("  direnv allow")

    print()
    print("Enter the environment with:")
    print("  nix develop")


def update_lock(directory):
    try:
        completed = subprocess.run(
            ["nix", "flake", "update"], check=False, cwd=directory
        )
    except OSError as error:
        raise PlainixError(
            f"Plainix: could not run `nix flake update`: {error}"
        ) from error
    if completed.returncode != 0:
        raise PlainixError(
            f"Plainix: `nix flake update` failed with exit code {completed.returncode}. "
            "See the Nix output above."
        )


def verify_systems(directory):
    print("Checking that every declared system still evaluates...", file=sys.stderr)
    try:
        completed = subprocess.run(
            [
                "nix",
                "eval",
                "--json",
                ".#devShells",
                "--apply",
                "shells: builtins.mapAttrs (_: shell: shell.default.drvPath) shells",
            ],
            check=False,
            cwd=directory,
            stdout=subprocess.DEVNULL,
        )
    except OSError as error:
        raise PlainixError(f"Plainix: could not run `nix eval`: {error}") from error
    if completed.returncode != 0:
        raise PlainixError(
            "Plainix: the updated environment does not evaluate. See the Nix output above."
        )


def read_lock(path):
    try:
        return path.read_bytes()
    except OSError as error:
        raise PlainixError(f"Plainix: could not read `{path}`: {error}") from error


def update(directory, run_update, verify):
    """Update flake.lock and return whether it changed, restoring it on any failure."""
    for file in ("plainix.nix", "flake.nix", "flake.lock"):
        if not (directory / file).is_file():
            raise PlainixError(
                f"Plainix: `{file}` not found; "
                "run this from a directory set up with `plainix init`"
            )

    lock_path = directory / "flake.lock"
    previous = read_lock(lock_path)
    try:
        run_update(directory)
        verify(directory)
    except BaseException as error:
        try:
            write_atomic(lock_path, previous)
        except PlainixError as restore_error:
            raise PlainixError(f"{error}\n{restore_error}") from error
        if isinstance(error, PlainixError):
            raise PlainixError(
                f"{error}\nPlainix: restored the previous `flake.lock`"
            ) from error
        raise
    return read_lock(lock_path) != previous


def build_parser():
    parser = argparse.ArgumentParser(
        prog="plainix",
        description="Plainix scaffolds reproducible project environments with Nix.",
        allow_abbrev=False,
    )
    parser.add_argument(
        "-V", "--version", action="version", version=f"plainix {VERSION}"
    )
    commands = parser.add_subparsers(dest="command")

    init = commands.add_parser(
        "init",
        allow_abbrev=False,
        help="create an environment in the current directory",
        description="Without options, Plainix opens an interactive tool selection. "
        "It generates .envrc for nix-direnv but never runs direnv allow.",
    )
    selection = init.add_mutually_exclusive_group()
    selection.add_argument(
        "--with",
        dest="tools",
        action="append",
        metavar="TOOL",
        help="include a tool without prompting; may be repeated",
    )
    selection.add_argument(
        "--empty",
        action="store_true",
        help="create an empty environment without prompting",
    )

    commands.add_parser(
        "update",
        allow_abbrev=False,
        help="update flake.lock, keeping it only if every declared system evaluates",
    )
    return parser


def run(arguments):
    parser = build_parser()
    options = parser.parse_args(arguments)
    directory = Path.cwd()

    if options.command == "update":
        if update(directory, update_lock, verify_systems):
            print("Updated flake.lock; every declared system still evaluates.")
            print("Review the changes above and commit flake.lock.")
        else:
            print("Already up to date.")
    elif options.command == "init":
        check_directory(directory)
        machine = os.uname()
        system = current_system(machine.machine, machine.sysname)
        if options.empty:
            ids = []
        elif options.tools:
            ids = options.tools
        else:
            ids = select_interactively(system)
        presets = resolve(ids, system)
        result = initialize(
            directory, presets, is_git_repository(directory), lock_flake
        )
        print_init_result(presets, result)
    else:
        parser.print_help()


def main():
    try:
        run(sys.argv[1:])
    except (PlainixError, OSError) as error:
        print(error, file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        return 130
    return 0


if __name__ == "__main__":
    sys.exit(main())
