{ pkgs, sampler }: pkgs.writeShellScriptBin "activate-sampler" ''
  mkdir -p /usr/local/bin/
  ln -sf ${sampler}/bin/sampler /usr/local/bin/
''
