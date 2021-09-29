# Note: We don't use Alpine and its packaged Rust/Cargo because they're too often out of date,
# preventing them from being used to build Substrate/Polkadot.

FROM phusion/baseimage:0.11 as builder
LABEL maintainer="chevdor@gmail.com"
LABEL description="This is the build stage for Substrate. Here we create the binary."

ENV DEBIAN_FRONTEND=noninteractive

ARG PROFILE=release
WORKDIR /deeper-chain

COPY . /deeper-chain

RUN apt-get update && \
	apt-get dist-upgrade -y -o Dpkg::Options::="--force-confold" && \
	apt-get install -y cmake pkg-config libssl-dev git clang

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
	export PATH="$PATH:$HOME/.cargo/bin" && \
	rustup toolchain install nightly && \
	rustup target add wasm32-unknown-unknown --toolchain nightly && \
	rustup target add wasm32-unknown-unknown && \
	rustup default stable && \
	cargo build "--$PROFILE"

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.11
LABEL maintainer="chevdor@gmail.com"
LABEL description="This is the 2nd stage: a very small image where we copy the deeper-chain binary."
ARG PROFILE=release

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	useradd -m -u 1000 -U -s /bin/sh -d /deeper-chain deeper-chain && \
	mkdir -p /deeper-chain/.local/share/deeper-chain && \
	chown -R deeper-chain:deeper-chain /deeper-chain/.local && \
	ln -s /deeper-chain/.local/share/deeper-chain /data

COPY --from=builder /deeper-chain/target/$PROFILE/deeper-chain /usr/local/bin
COPY --from=builder /deeper-chain/target/$PROFILE/node-rpc-client /usr/local/bin
COPY --from=builder /deeper-chain/target/$PROFILE/node-bench /usr/local/bin

# checks
RUN ldd /usr/local/bin/deeper-chain && \
	/usr/local/bin/deeper-chain --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
	rm -rf /usr/bin /usr/sbin /usr/share/man

USER deeper-chain
EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

CMD ["/usr/local/bin/deeper-chain"]