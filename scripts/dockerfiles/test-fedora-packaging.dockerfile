FROM fedora:41

ARG PG_MAJOR=16
ENV PG_MAJOR=${PG_MAJOR}

# Install PostgreSQL from PGDG and pgvector
RUN dnf -y install dnf-plugins-core wget gnupg2 ca-certificates
RUN dnf -y install https://download.postgresql.org/pub/repos/yum/reporpms/F-41-x86_64/pgdg-fedora-repo-latest.noarch.rpm
RUN dnf -y install postgresql${PG_MAJOR} postgresql${PG_MAJOR}-server postgresql${PG_MAJOR}-contrib pgvector_${PG_MAJOR}

RUN mkdir -p /rpms
COPY target/packages/*.rpm /rpms
RUN dnf -y install /rpms/*.rpm && dnf clean all

COPY scripts/package-test-redhat.sh /package-test.sh
RUN chmod +x /package-test.sh
RUN /package-test.sh

CMD ["bash"]