# CSGO ESP (for learning Rust)

`This shouldn't be used for online gameplay, you will probably get banned, and if you do decide to use this online and get banned I am not responsible for the loss of your account.`

Credit to https://github.com/a2x/cs2-dumper for the offsets generator.

# Demo

(https://daryan-public.s3.eu-west-2.amazonaws.com/random/csgo_testing.mp4)

## Motivation

This project was to improve my rust skills as well as to understand some of the basics of external game hacking.

## How it works

The project works by using tauri, a rust framework which uses the native OS web viewer + a rust backend that communicates with the native web viewer through IPC.

Using Tauri, the backend rust service would open a connection to the running csgo processes and use the Windows API to read the process memory. Through this I can get the players position and the users camera position.

With this information I used a world to screen function, which I honestly don't entirely understand, but it essentially maps 3d co ordinates to your 2d monitor. With this information I would pass it to the web viewer, which would render it on screen.

The web viewer is a top most window that can't be focused and is transparent, with this it can overlay the running game window.
