# ASTRA.OS Build Environment
FROM ubuntu:22.04

# Avoid interactive prompts
ENV DEBIAN_FRONTEND=noninteractive

# Install Buildroot dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    gcc \
    g++ \
    make \
    patch \
    wget \
    cpio \
    unzip \
    rsync \
    bc \
    bzip2 \
    xz-utils \
    file \
    libncurses-dev \
    python3 \
    python3-pip \
    perl \
    git \
    locales \
    && rm -rf /var/lib/apt/lists/*

# Set locale
RUN locale-gen en_US.UTF-8
ENV LANG=en_US.UTF-8
ENV LC_ALL=en_US.UTF-8

# Create build user (Buildroot doesn't like running as root)
RUN useradd -m -s /bin/bash builder
RUN mkdir -p /astra && chown builder:builder /astra

# Set working directory
WORKDIR /astra

# Switch to builder user
USER builder

# Default command
CMD ["/bin/bash"]
