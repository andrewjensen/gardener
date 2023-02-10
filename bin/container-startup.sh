#!/bin/bash

# This script is the entrypoint for the server when run within the docker image.

set -euo pipefail

# TODO: find a way to get install.sh to work in the docker image layers so we
# don't need to invoke it on startup
cd /code/lib/pd2dsy/
./install.sh

cd /code/
./gardener
