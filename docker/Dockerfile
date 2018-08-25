FROM fedora:28

RUN dnf -y install \
    gcc

ENV HOME="/root"

# Install Rust
RUN curl https://sh.rustup.rs -sSf > install.sh
RUN sh install.sh -y && rm -f install.sh
RUN echo "source $HOME/.cargo/env" >> /etc/bashrc
ENV PATH "/root/.cargo/bin:$PATH"

# Windows-related configuration
RUN rustup target add x86_64-pc-windows-gnu
RUN dnf -y install \
    mingw64-{gcc,winpthreads-static} \
    p7zip{,-plugins} \
    wget

RUN mkdir -p $HOME/libusb && cd $HOME/libusb && \
    wget https://github.com/libusb/libusb/releases/download/v1.0.22/libusb-1.0.22.7z && \
    7z x libusb-1.0.22.7z

COPY libusb.pc $HOME/libusb/libusb-1.0.pc

ENV CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
ENV CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
ENV CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++

# Linux-related configuration
RUN dnf -y install libusb-devel
RUN ln -sf /usr/lib64/libusb.so /usr/lib64/usb-1.0

VOLUME ["/build"]

# Define working directory.
WORKDIR /build

CMD ["bash", "/bin/bash"]