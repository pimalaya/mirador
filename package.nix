# TODO: move this to nixpkgs
# This file aims to be a replacement for the nixpkgs derivation.

{ lib
, pkg-config
, rustPlatform
, fetchFromGitHub
, stdenv
, apple-sdk
, installShellFiles
, installShellCompletions ? stdenv.buildPlatform.canExecute stdenv.hostPlatform
, installManPages ? stdenv.buildPlatform.canExecute stdenv.hostPlatform
, buildNoDefaultFeatures ? false
, buildFeatures ? [ ]
}:

let
  version = "1.0.0";
  hash = "";
  cargoHash = "";
in

rustPlatform.buildRustPackage {
  inherit cargoHash version;
  inherit buildNoDefaultFeatures buildFeatures;

  pname = "mirador";

  src = fetchFromGitHub {
    inherit hash;
    owner = "pimalaya";
    repo = "mirador";
    rev = "v${version}";
  };

  nativeBuildInputs = [ pkg-config ]
    ++ lib.optional (installManPages || installShellCompletions) installShellFiles;

  buildInputs =
    lib.optional stdenv.hostPlatform.isDarwin apple-sdk;

  doCheck = false;
  auditable = false;

  # unit tests only
  cargoTestFlags = [ "--lib" ];

  postInstall = ''
    mkdir -p $out/share/{services,completions,man}
    cp assets/mirador@.service "$out"/share/services/
  '' + lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
    "$out"/bin/mirador man "$out"/share/man
  '' + lib.optionalString installManPages ''
    installManPage "$out"/share/man/*
  '' + lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
    "$out"/bin/mirador completion bash > "$out"/share/completions/mirador.bash
    "$out"/bin/mirador completion elvish > "$out"/share/completions/mirador.elvish
    "$out"/bin/mirador completion fish > "$out"/share/completions/mirador.fish
    "$out"/bin/mirador completion powershell > "$out"/share/completions/mirador.powershell
    "$out"/bin/mirador completion zsh > "$out"/share/completions/mirador.zsh
  '' + lib.optionalString installShellCompletions ''
    installShellCompletion "$out"/share/completions/mirador.{bash,fish,zsh}
  '';

  meta = {
    description = "CLI to watch mailbox changes";
    mainProgram = "mirador";
    homepage = "https://github.com/pimalaya/mirador/";
    changelog = "https://github.com/soywod/mirador/blob/v${version}/CHANGELOG.md";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ soywod ];
  };
}
