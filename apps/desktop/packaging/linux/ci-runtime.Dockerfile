# Pinned Ubuntu 24.04 image index; only runtime OS packages and Xvfb test tools.
FROM ubuntu:24.04@sha256:534baea6a22c03a63003dbc8dbe78fe34bc0d7e595d9a9dc9834884ff530eb55
ARG XDZIP_RUNTIME_PACKAGES
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    ${XDZIP_RUNTIME_PACKAGES} libssl3t64 ca-certificates xvfb xauth fonts-noto-cjk locales desktop-file-utils shared-mime-info xdg-utils libglib2.0-bin python3 pkexec \
    && locale-gen zh_CN.UTF-8 en_US.UTF-8 \
    && rm -rf /var/lib/apt/lists/*
