# cef

A high level crate for the Chromium Embedded Framework. See
(cef-sys)[https://github.com/anlumo/cef-sys] for more.

## Running examples

Some platform specific setup is required to make the examples runnable.

### Windows

### Linux

### macOS

// TODO(yanchith): audit
- Copy the contents of `Chromium Embedded
  Framework.framework/Libraries` to `/usr/lib` or `/usr/local/lib`
- Copy the `Chromium Embedded Framework.framework` directory to this
  directory
- Copy the `libswiftshader_libEGL.dylib` and
  `libswiftshader_libGLESv2.dylib` from `Chromium Embedded
  Framework.framework/Libraries` to `./target/debug/examples/` in this
  directory
