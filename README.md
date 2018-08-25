
# Table of Contents

1.  [Fastboot-rs](#orgeb7b043)
    1.  [Fastboot](#org06bbc39)
    2.  [Dependencies](#org1b2ea3c)
    3.  [Compiling](#org51d4cf8)
    4.  [Cross-compiling](#org8069c3b)
    5.  [What's next?](#orgd342ab5)


<a id="orgeb7b043"></a>

# Fastboot-rs


<a id="org06bbc39"></a>

## Fastboot

This crate provides a simple implementation of Android Fastboot Protocol in form of Rust Trait. To demonstrate how it can actually work I used [Libusb](http:libusb.info) 
to communicate with devices. Note, this project is just my Rust playground and has a lot to be improved. 


<a id="org1b2ea3c"></a>

## Dependencies

The trait itself doesn't have any extra dependencies. However, if you want to try the examples you must have
***libusb*** installed.


<a id="org51d4cf8"></a>

## Compiling

This library is written in Rust so to compile it you need Rust and Cargo installed.


<a id="org8069c3b"></a>

## Cross-compiling

To cross-compile examples you will have to use Docker. To do that:

-   Build a docker image:

    `docker build -t "<name>" -f ./docker/Dockerfile ./docker`

-   Run compilation for supported targets. For Windows, for example:

    `docker run --rm -v "$(pwd)":/build/ "<name>" 
    sh -c "PKG_CONFIG_PATH=$HOME/libusb PKG_CONFIG_ALLOW_CROSS=1 
    cargo build --target=x86_64-pc-windows-gnu --examples"`


<a id="orgd342ab5"></a>

## What's next?

-   Better tests;
-   More idiomatic Rust code;
-   Split the actual trait implementation.

