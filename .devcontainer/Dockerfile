FROM aarch64/ubuntu
WORKDIR /rustspot

RUN DEBIAN_FRONTEND=noninteractive apt update && apt install -y \
    build-essential \
    curl \
    git \
    cmake

RUN apt-get update

RUN git clone https://github.com/AreaScout/SDL.git
RUN cmake -H"SDL" -B"build"
RUN cmake --build build --target install -- -j8

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"
