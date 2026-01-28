FROM ubuntu:latest AS builder

RUN apt update && apt install -y \
    net-tools \
    openssl \
    dovecot-core \
    dovecot-pop3d

RUN useradd -p $(openssl passwd -1 user) --create-home user

COPY conf/* /etc/dovecot/conf.d

COPY --chown=user:mail mail/dev_perl_apache_org_2000-09.mbox /var/mail/user

