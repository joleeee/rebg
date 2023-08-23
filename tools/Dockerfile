FROM ubuntu

# dependencies
RUN apt-get update
RUN apt-get install -y build-essential git python3 ninja-build pkg-config libglib2.0-dev libpixman-1-dev flex bison

# libcs
RUN apt-get install -y \
    libc6-dbg-arm64-cross \
    libc6-dbg-amd64-cross \
    libc6-dbg \
    libc6-amd64-i386-cross \
    libc6-amd64-x32-cross

# get source
COPY qemu /root/qemu

# compile
WORKDIR /root/qemu
RUN ./configure --with-git-submodules=ignore --enable-tcg-interpreter --target-list=aarch64-linux-user,x86_64-linux-user
RUN make -j$(nproc)
RUN make install

# clean up
WORKDIR /root
RUN rm -r qemu

# stick around
ENTRYPOINT ["sleep", "infinity"]