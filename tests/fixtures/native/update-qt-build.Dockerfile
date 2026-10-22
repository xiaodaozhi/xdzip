# tests/fixtures/native/update-qt-build.Dockerfile

FROM xdzip-update-regression
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake ninja-build patchelf pkg-config python3-venv git \
    libgl1-mesa-dev libegl1-mesa-dev libxkbcommon-dev libxkbcommon-x11-0 \
    libxcb1 libxcb-cursor0 libxcb-icccm4 libxcb-image0 libxcb-keysyms1 \
    libxcb-render0 libxcb-render-util0 libxcb-shape0 libxcb-randr0 \
    libxcb-xfixes0 libxcb-sync1 libxcb-shm0 libxcb-util1 libxcb-xinerama0 libxcb-xkb1 \
    libx11-dev libx11-6 libx11-xcb1 libxrender1 libxext6 libxi6 libxfixes3 \
    libxrandr2 libfontconfig1 libfreetype6 libice6 libsm6 \
    && rm -rf /var/lib/apt/lists/*
RUN python3 -m venv /opt/aqt \
    && /opt/aqt/bin/pip install aqtinstall==3.3.0 \
    && /opt/aqt/bin/python -m aqt install-qt linux desktop 6.8.2 linux_gcc_64 --outputdir /opt/Qt
