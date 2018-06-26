# I use this Dockerfile to build linux Snap files from OSX.

FROM ubuntu:xenial

# Enable multiverse as snapcraft cleanbuild does.
RUN sed -i 's/ universe/ universe multiverse/' /etc/apt/sources.list

RUN apt-get update && \
  apt-get dist-upgrade --yes && \
  apt-get install --yes \
  git \
  snapcraft \
  && \
  apt-get autoclean --yes && \
  apt-get clean --yes

# Required by click.
ENV LC_ALL C.UTF-8
ENV SNAPCRAFT_SETUP_CORE 1

# update Cargo.toml version
# update snapcraft.yaml version
# $ docker build -t snap .
# $ docker run -it -v `pwd`:./derp snap /bin/bash
# cd /tmp
# apt-get install curl
# curl https://sh.rustup.rs -sSf | sh
# source $HOME/.cargo/env
# snapcraft
# snapcraft push *.snap
# snapcraft release loop-rs 1 beta 
