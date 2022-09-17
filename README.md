<div align="center">
  <h1>Topaz 💎</h1>
  <p>Lightweight util for handling animated wallpapers under X11 on Arch.</p>
  <img src="https://user-images.githubusercontent.com/34040324/190539508-9507ac4d-14cd-416b-81c5-f8d3e40db433.gif" height="360px" /><br /><br />
</div>

It's just a simple bash script for now - it wraps xwinwrap and provides a live-reload config, plus basic CLI args.

Eventually, I'd like to expand on it & add some GUI config options.

## Installation
For now, just move the `topaz` script to `/usr/bin`, add it to your autostart (e.g. `bspwmrc`), and create a config file at `~/.config/topaz/topaz.conf`. The image/video to use can be provided in the config in the format `file=/path/goes/here.gif`, and any additional options you'd like to pass to mpv as `mpvoptions="--opts --here"`

## Dependancies
- [xwinwrap-git](https://aur.archlinux.org/packages/xwinwrap-git)
- [mpv](https://archlinux.org/packages/community/x86_64/mpv/)

