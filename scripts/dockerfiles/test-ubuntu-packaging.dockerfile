FROM ubuntu:22.04

ARG PG_MAJOR=16
ENV DEBIAN_FRONTEND=noninteractive
ENV PG_MAJOR=${PG_MAJOR}

RUN apt-get update && \
    apt-get install -y wget gnupg lsb-release && \
    sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list' && \
    wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | apt-key add - && \
    apt-get update && \
    apt-get install -y postgresql-${PG_MAJOR} postgresql-contrib-${PG_MAJOR} postgresql-${PG_MAJOR}-pgvector

RUN mkdir -p /debs
COPY target/packages/*.deb /debs
RUN dpkg -i /debs/*.deb

COPY scripts/package-test.sh /package-test.sh
RUN chmod +x /package-test.sh
RUN /package-test.sh

CMD ["bash"]
