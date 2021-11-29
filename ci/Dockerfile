FROM aarch64/ubuntu
WORKDIR /rustspot

RUN DEBIAN_FRONTEND=noninteractive apt update
RUN apt install -y build-essential
RUN apt install -y curl
RUN apt install -y git
RUN apt install -y cmake

RUN git clone https://github.com/AreaScout/SDL.git dep/sdl
RUN cmake -H"dep/sdl" -B"target/sdl"
RUN cmake --build target/sdl --target install -- -j8

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Build dependencies only
RUN cargo init
COPY ci/cargo.sh ci/cargo.sh
COPY Cargo.toml Cargo.toml
RUN bash ci/cargo.sh && cat Cargo.toml
RUN cargo build --release --features "gles"

# Build whole project
COPY Cargo.toml Cargo.toml
COPY src src

RUN cargo build --release --features "gles"

# Allows running cp in docker run
ENTRYPOINT ["/bin/bash", "-l", "-c"]

# Fix "mesg: ttyname failed: Inappropriate ioctl for device" warning
RUN cp /etc/profile /root/.profile
