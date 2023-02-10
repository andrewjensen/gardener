# temporary image to compile Rust
FROM rust:1.67-slim-buster AS cargo
WORKDIR /code
COPY . .
RUN cargo build --release

# start a new image with only runtime dependencies
FROM debian:stable AS release

# copy compiled Rust project into the new image
WORKDIR /code
COPY --from=cargo /code/target/release/gardener /code

# copy other assets
COPY --from=cargo /code/bin       /code/bin
COPY --from=cargo /code/public    /code/public
COPY --from=cargo /code/workspace /code/workspace

# add system dependencies
RUN apt update && apt install -y \
  build-essential \
  git \
  wget \
  python3-pip \
  python3-venv

RUN mkdir lib

# add lib: pd2dsy
RUN git clone \
  --recurse-submodules \
  https://github.com/electro-smith/pd2dsy.git \
  lib/pd2dsy
RUN cd lib/pd2dsy \
  && pip3 install -r requirements.txt \
  && ./install.sh

# Adding virtualenv binaries to the system path manually, see:
# https://pythonspeed.com/articles/activate-virtualenv-dockerfile/
ENV PATH="/code/lib/pd2dsy/pd_env/bin/:${PATH}"

# add lib: arm toolchain
# Found on this page:
# https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads
# "x86_64 Linux hosted cross toolchains" > "AArch32 bare-metal target (arm-none-eabi)"
RUN cd /code/lib \
  && wget https://developer.arm.com/-/media/Files/downloads/gnu/12.2.rel1/binrel/arm-gnu-toolchain-12.2.rel1-x86_64-arm-none-eabi.tar.xz \
  && tar -xf arm-gnu-toolchain-12.2.rel1-x86_64-arm-none-eabi.tar.xz \
  && rm arm-gnu-toolchain-12.2.rel1-x86_64-arm-none-eabi.tar.xz
ENV PATH="${PATH}:/code/lib/arm-gnu-toolchain-12.2.rel1-x86_64-arm-none-eabi/bin"
