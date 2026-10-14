FROM php:8.3-apache

RUN apt-get update \
    && apt-get install -y --no-install-recommends libsqlite3-dev libldap2-dev libxml2-dev \
    && docker-php-ext-install pdo_sqlite pdo_mysql ldap dom \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /var/www/html
COPY src/ ./
COPY database/ /var/www/database/
COPY LICENSE /usr/local/share/project-license.txt

RUN mkdir -p /var/www/uploads \
    && ln -s /var/www/uploads /var/www/html/uploads \
    && chown -R www-data:www-data /var/www/database /var/www/uploads

EXPOSE 80
