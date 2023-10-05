# image for iteratively compiling qemu, only has dependencies
# you should mount a copy of qemu somewhere

FROM ubuntu

# dependencies
RUN apt-get update
RUN apt-get install -y build-essential git python3 ninja-build pkg-config libglib2.0-dev libpixman-1-dev flex bison

# dev tools
RUN apt-get install -y neovim netcat-openbsd

WORKDIR /root/qemu

# stick around
ENTRYPOINT ["sleep", "infinity"]