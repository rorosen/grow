# Grow Agent

The grow agent is developed to run on a Raspberry Pi.

## Pi Pre-Install

Flash the image

```bash
unxz -c /path/to/image.img.xz | sudo dd of=/dev/sda bs=4M
```

Mount the boot partition of the image and place the `ssh` file

```bash
udisksctl mount -b /dev/sda1
touch /run/media/rob/bootfs/ssh
```

Hash the password and create the `userconf.txt` file

```bash
PASS=$(openssl passwd -6)
echo "rob:$PASS" > /run/media/rob/bootfs/userconf.txt
```

Unmount the boot partition

```bash
udisksctl unmount -b /dev/sda1
```

## Set up the Pi

Multi-user nix installation

```bash
curl -L https://nixos.org/nix/install | sh -s -- --daemon
```

Comment default secure path with visudo

```bash
echo 's/^Defaults[[:blank:]]secure_path=/#&/' | (sudo su -c 'EDITOR="sed -f- -ie" visudo')
```

Source the Nix profile at the beginning of `~/.bashrc` before anything else

```bash
sed -i '1s|^|if [ -e "/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh" ]; then\n    . "/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh"\nfi\n|' ~/.bashrc
```

Add `rob` to the trusted Nix users

```bash
echo "trusted-users = rob" | sudo tee -a /etc/nix/nix.conf >/dev/null
sudo systemctl restart nix-daemon
```

Set hostname and reboot

```bash
sudo hostnamectl set-hostname growPi
sudo reboot
```

## Deploy the Systemd Service

```bash
nix run github:serokell/deploy-rs .#growPi
```
