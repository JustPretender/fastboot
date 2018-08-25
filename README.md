
# Table of Contents

1.  [Fastboot-rs](#org90890f0)
    1.  [Fastboot](#orgbb7e011)
    2.  [Dependencies](#org4e0feeb)
    3.  [Compiling](#orgb0ea037)
    4.  [Cross-compiling](#orgd2c83e6)
    5.  [To be improved](#org438e7b8)


<a id="org90890f0"></a>

# Fastboot-rs


<a id="orgbb7e011"></a>

## Fastboot

This crate provides a simple implementation of Android Fastboot Protocol in form of Rust Trait. To demonstrate how it can actually work I used [Libusb](http:libusb.info) 
to communicate with devices. Note, this project is just my Rust playground and has a lot to be improved. 


<a id="org4e0feeb"></a>

## Dependencies

The trait itself doesn't have any extra dependencies. However, if you want to try the examples you must have
***libusb*** installed.


<a id="orgb0ea037"></a>

## Compiling

This library is written in Rust so to compile it you need Rust and Cargo installed.


<a id="orgd2c83e6"></a>

## Cross-compiling

One can build a docker image, which can be used for cross-compilation for Windows and Linux
using a Dockerfile located in *./docker*


<a id="org438e7b8"></a>

## To be improved

-   Better tests;
-   More idiomatic Rust code;
-   Split the actual trait implementation.

