FROM ubuntu

# dependencies
RUN apt-get update
RUN apt-get install -y build-essential git python3 ninja-build pkg-config libglib2.0-dev libpixman-1-dev flex bison

# get source
COPY qemu /root/qemu
RUN chown root:root -R /root/qemu

# compile
WORKDIR /root/qemu
RUN ./configure --with-git-submodules=ignore --enable-tcg-interpreter --cpu=unknown --target-list=aarch64-linux-user,x86_64-linux-user
RUN make -j4
RUN make install

# clean up
WORKDIR /root
RUN rm -r qemu

# stick around
ENTRYPOINT ["sleep", "infinity"]