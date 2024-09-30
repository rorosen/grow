# Nix SD Image

This example shows how to build a pre-configured image for a Raspberry Pi 4 that you can flash to an
SD card and run right away.

You may want to place your SSH key in the [configuration](./configuration.nix) to get access to the
system. If you don't have Nix installed already, check out the
[Determinate Nix Installer](https://determinate.systems/posts/determinate-nix-installer/).

The configuration uses the `-aarch64` packages in order to enable cross-compiling from `x86_64`,
which is faster than emulating `aarch64-linux` for the Rust build. However, the rest of the build is
emulated so we can use the Nix binary cache. If you are on `x86_64` and want to emulate an
`aarch64-linux` system, you need to enable [binfmt](https://en.wikipedia.org/wiki/Binfmt_misc) for
that system. On NixOS this can be done via the `boot.binfmt.emulatedSystems` option.

```nix
boot.binfmt.emulatedSystems = [ "aarch64-linux" ];
```

```shell
# Build the image
nix build .\#packages.aarch64-linux.sdImage
# Flash it to an SD card, replace sdX
sudo dd if=$(ls result/sd-image/nixos-sd-image-*-aarch64-linux.img) of=/dev/sdX bs=4M
```

Once the Pi is up and running, you can visit the Grafana instance on port 80 of the Pi. Log in with
the default credentials (username: `admin`, password: `admin`) and navigate to `Dashboards` ->
`Grow` in the menu.

You can make changes to the configuration and remotely rebuild the system, if you placed an SSH key
in the configuration.

```shell
# Rebuild the configuration, replace the IP address
nixos-rebuild switch --flake .\#pi --target-host root@192.168.123.123
```
