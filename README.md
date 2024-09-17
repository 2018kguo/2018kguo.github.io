Personal website hosted on github pages. Used to be a gatsby.js static site, now it uses egui/eframe (powered by Rust & WASM)

Refer to https://github.com/emilk/eframe_template for directions on how to get something like this set up

Some modifications I made to get this working on github pages:
 - removed the service workers/PWA stuff
 - removed the public_url env var from the pages github action (chrome wasn't happy w/ the generated links for whatever reason)