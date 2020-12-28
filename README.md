# AWS Runtime (for Rust)
This project is intended to be a mini library capable of handling AWS Lambda invocations. Since it is written in rust, there is some boilerplate needed in order for a rust application to play nicely with AWS. Solving that boilerplate is the aim of this project.

## Motivation
The motivation for this project is a simple one: I just wanted to run some rust code in a Lambda. As it turns out, this is not natively supported but thankfully it isn't terribly difficult and the lambda runtime process is well documented.

# Compilation
There are some things to do before we can actually compile. You need to setup your environment (as shown below) or else the requisite commands/binary will be incompatible with Amazon Linux.

## Compilation Enviornment
Our program ultimately must be cross-compiled in a way that is able to be run on an ec2 container in the aether. To achieve this, we'll need to first install the correct system target.

``` bash
rustup target add x86_64-unknown-linux-musl
```

This references a thing called **linux-musl** which is a specific toolchain compatible with *amazonlinux* instances. And speaking of... you probably need to download it! You can download a [fresh version](https://git.musl-libc.org/cgit/musl/) if you don't want to use my old-school cool src.

``` bash
cd ~/
wget https://git.musl-libc.org/cgit/musl/snapshot/musl-1.2.1.tar.gz
tar -xsvf musl-1.2.1.tar.gz
cd musl-1.2.1
./configure
make
sudo make install
```

After running that last command, you will need to update your PATH variable. Assuming you have bash... add this to the end of your **~/.profile** file.

``` bash
export PATH="/usr/local/musl/bin:$PATH"
```

You are now ready to cross-compile!

The first thing that rust and AWS think of when they hear **http web request** is apparently *openSSL*. It wouldn't be my first choice, but here we are. To make matters worse, the specific RHEL flavor of container that lambdas run on does not play nicely with the newest openSSL version. This means we need to build and link our lambda using the downgraded 1.0.1k branch.

``` bash
cd ~/
wget https://www.openssl.org/source/old/1.0.1/openssl-1.0.1k.tar.gz
tar -xvf openssl-1.0.1k.tar.gz
cd openssl-1.0.1k
./config
make
sudo make install
```

After running that last command, you will need to update your PATH variable. Assuming you have bash... add this to the end of your **~/.profile** file.

``` bash
export PATH="/usr/local/ssl/bin:$PATH"
```

## Compilation
An example makefile. Just put all this stuff on one line and you'll be golden.
```bash
build:
	OPENSSL_DIR=/usr/local/ssl \
    PKG_CONFIG_ALLOW_CROSS=1 \
    cargo build --release --target x86_64-unknown-linux-musl
```

## Final Deliverable
The finished directory that we create will ultimately look like so:
<div style="padding-left: 30px">
| function
<br/>|&nbsp;-&nbsp;bootstrap.sh
<br/>|&nbsp;-&nbsp;binary
</div>


Before our lambda can be executed it needs an entrypoint to orchestrate everything. This entrypoint needs to be a file called **bootstrap.sh** which is executable.

**bootstrap.sh**
```bash
#!/bin/sh
cd $LAMBDA_TASK_ROOT
./binary
```

Note: **binary** is simply the final rust executable.

You can go ahead and zip all this up now

```bash
    zip -j function.zip ./function/*
```

# License
MIT
