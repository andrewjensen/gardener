## About

### What's Gardener?

Gardener is a website that lets you convert a [Pure Data](https://puredata.info/) patch for [Daisy](https://www.electro-smith.com/daisy) hardware, so you can create synths and effects without needing to write any code. It's powered by the excellent [pd2dsy](https://github.com/electro-smith/pd2dsy) program. pd2dsy can be confusing to install and set up, so Gardener runs pd2dsy in the cloud for you.

Gardener is a project by [Andrew Jensen](https://andrewjensen.io/).

### How does it work? (For fellow nerds)

Gardener is written in Rust on the [actix-web](https://actix.rs/) framework. When a user uploads a Pure Data patch, the server saves that file to disk and adds its ID into a queue. Then a background worker takes the item from the queue, calls pd2dsy to generate C++ code from the patch, compiles the code into a binary file, and moves that into a directory of downloads.

Gardener is open source software, so you can host your own copy if you want! You can find the [gardener repo](https://github.com/andrewjensen/gardener) on Github.
