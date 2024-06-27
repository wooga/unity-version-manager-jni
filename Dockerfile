#Using older version of debian (stretch) to use older glibc, and thus with wider compatibility range.
FROM openjdk:8-jdk-stretch
ARG RUST_VERSION=1.50.0
ARG UVM_VERSION=2.2.0

# Create an app user so our program doesn't run as root.
# Set the home directory to our app user's home.

ENV RUST_BACKTRACE=1
ENV RUST_LOG="warning, uvm_core=trace, uvm_jni=trace"
ENV IN_DOCKER="1"

RUN echo "deb http://archive.debian.org/debian stretch main contrib non-free" > /etc/apt/sources.list
RUN apt-get update
RUN apt-get install -y make build-essential libssl-dev pkg-config openssl p7zip-full cpio

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain $RUST_VERSION
ENV PATH="${HOME}/.cargo/bin:${PATH}"
RUN curl -Lo "unity-version-manager-$UVM_VERSION.tar.gz" "https://github.com/Larusso/unity-version-manager/archive/v$UVM_VERSION.tar.gz"
RUN tar -xzf "unity-version-manager-$UVM_VERSION.tar.gz" && rm -f "unity-version-manager-$UVM_VERSION.tar.gz"
RUN cd "unity-version-manager-$UVM_VERSION" && PATH="${HOME}/.cargo/bin:${PATH}" make install

ARG USER_ID=1001
ARG GROUP_ID=100
RUN useradd -u ${USER_ID} -g ${GROUP_ID} --create-home jenkins_agent

USER jenkins_agent
RUN uvm install 2019.1.0a7 /home/jenkins_agent/.local/share/Unity-2019.1.0a7
