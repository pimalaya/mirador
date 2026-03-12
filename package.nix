# TODO: move this to nixpkgs
# This file aims to be a replacement for the nixpkgs derivation.

{
  buildFeatures ? [ ],
  buildNoDefaultFeatures ? false,
  buildPackages,
  fetchFromGitHub,
  installManPages ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  installShellCompletions ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  installShellFiles,
  lib,
  openssl,
  pkg-config,
  rustPlatform,
  stdenv,
}:

let
  version = "0.1.0";
  hash = "";
  cargoHash = "";

  emulator = stdenv.hostPlatform.emulator buildPackages;
  exe = stdenv.hostPlatform.extensions.executable;

in
rustPlatform.buildRustPackage {
  inherit
    cargoHash
    version
    buildNoDefaultFeatures
    buildFeatures
    ;

  pname = "mirador";

  src = fetchFromGitHub {
    inherit hash;
    owner = "pimalaya";
    repo = "mirador";
    rev = "v${version}";
  };

  env = {
    # OpenSSL should not be provided by vendors, not even on Windows
    OPENSSL_NO_VENDOR = "1";
  };

  nativeBuildInputs = [
    pkg-config
  ] ++ lib.optional (installManPages || installShellCompletions) installShellFiles;

  buildInputs = lib.optional (builtins.elem "native-tls" buildFeatures) openssl;

  doCheck = false;

  postInstall =
    lib.optionalString (lib.hasInfix "wine" emulator) ''
      export WINEPREFIX="''${WINEPREFIX:-$(mktemp -d)}"
      mkdir -p $WINEPREFIX
    ''
    + ''
      mkdir -p $out/share/{completions,man}
      ${emulator} "$out"/bin/mirador${exe} manuals "$out"/share/man
      ${emulator} "$out"/bin/mirador${exe} completions -d "$out"/share/completions bash elvish fish powershell zsh
    ''
    + lib.optionalString installManPages ''
      installManPage "$out"/share/man/*
    ''
    + lib.optionalString installShellCompletions ''
      installShellCompletion --cmd mirador \
        --bash "$out"/share/completions/mirador.bash \
        --fish "$out"/share/completions/mirador.fish \
        --zsh "$out"/share/completions/_mirador
    '';

  meta = {
    description = "CLI to watch mailbox changes";
    mainProgram = "mirador";
    homepage = "https://github.com/pimalaya/mirador";
    changelog = "https://github.com/pimalaya/mirador/blob/v${version}/CHANGELOG.md";
    license = lib.licenses.agpl3Plus;
    maintainers = with lib.maintainers; [ soywod ];
  };
}
