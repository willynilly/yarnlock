{ isDevelopment ? true }:

let
  # Update packages with `nixpkgs-update` command
  pkgs = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/507b63021ada5fee621b6ca371c4fca9ca46f52c.tar.gz") { };

  packages' = with pkgs; [
    coreutils
    python313
    uv
    ruff
    maturin
    (lib.optional stdenv.isDarwin libiconv)

    (writeShellScriptBin "make" "maturin develop --uv")
    (writeShellScriptBin "run-tests" ''
      python -m pytest . \
        --verbose \
        --no-header
    '')
    (writeShellScriptBin "nixpkgs-update" ''
      hash=$(
        curl --silent --location \
        https://prometheus.nixos.org/api/v1/query \
        -d "query=channel_revision{channel=\"nixpkgs-unstable\"}" | \
        grep --only-matching --extended-regexp "[0-9a-f]{40}")
      sed -i -E "s|/nixpkgs/archive/[0-9a-f]{40}\.tar\.gz|/nixpkgs/archive/$hash.tar.gz|" shell.nix
      echo "Nixpkgs updated to $hash"
    '')
  ];

  shell' = with pkgs; lib.optionalString isDevelopment ''
    export PYTHONNOUSERSITE=1
    export PYTHONPATH=""
    export TZ=UTC

    current_python=$(readlink -e .venv/bin/python || echo "")
    current_python=''${current_python%/bin/*}
    [ "$current_python" != "${python313}" ] && rm -rf .venv/

    echo "Installing Python dependencies"
    export UV_PYTHON="${python313}/bin/python"
    NIX_ENFORCE_PURITY=0 uv sync --frozen

    echo "Activating Python virtual environment"
    source .venv/bin/activate
  '';
in
pkgs.mkShell {
  buildInputs = packages';
  shellHook = shell';
}
