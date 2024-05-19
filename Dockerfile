FROM rust:latest

ARG TLS="1"

# Possible values: CLIENT, SERVER1, SERVER2 and BOOTSTRAP
# Defines how the node will behave
ENV EXEC_MODE="BOOTSTRAP"

# If set to "y" will just use 127.0.0.1 as the bootstrap node
ENV DEFAULT_BOOTSTRAP="y"

# Used to define paths, if set to windows \\ will be used instead of /
ENV OS_CONF="linux"


# Install dependencies
RUN apt-get update && \
    apt-get install -y protobuf-compiler

# Verify that protoc is installed
RUN protoc --version

# Set the working directory
WORKDIR /usr/src/

# Copy the current directory contents into the container at /usr/src/myapp
COPY . .

RUN rm -drf cert/*

# Copy the CA key and certificate into the container
COPY cert/ca.key cert/ca.key
COPY cert/ca.crt cert/ca.crt

COPY cert/server.cnf cert/server.cnf
COPY cert/ca.cnf cert/ca.cnf

WORKDIR /usr/src/cert

# Create necessary directory structure for OpenSSL
RUN mkdir -p demoCA/certs demoCA/newcerts demoCA/private && \
    touch demoCA/index.txt && \
    echo 1000 > demoCA/serial

# Generate the server private key
RUN openssl genpkey -algorithm RSA -out server.key

# Generate the server CSR
RUN openssl req -new -key server.key -out server.csr -config server.cnf

# Sign the server certificate with the CA
RUN openssl ca -batch -config ca.cnf -extensions v3_req -days 375 -notext -md sha256 -in server.csr -out server.crt

# Remove the CA key to ensure it's not in the final image
RUN rm ca.key

WORKDIR /usr/src/

# Add the default bootstrap list
RUN echo "127.0.0.1" >> ./src/bootstrap.txt && echo >> ./src/bootstrap.txt

RUN cargo build --release

CMD ["./target/release/public_ledger", "bootstrap"]
