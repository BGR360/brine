FROM gradle:jdk17

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive \
    apt-get install -y \
    git \
    python3 \
    vim \
    && apt-get clean

RUN git clone https://github.com/hube12/DecompilerMC.git

WORKDIR /home/gradle/DecompilerMC/

COPY ./scripts/decompile-and-copy.sh .
RUN chmod +x ./decompile-and-copy.sh

CMD ["python3", "main.py", "--help"]