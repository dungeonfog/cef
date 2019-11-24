# cef

A high level crate for the Chromium Embedded Framework. See
(cef-sys)[https://github.com/anlumo/cef-sys] for more.

## Running examples

Some platform specific setup is required to make the examples runnable.

### Windows

### Linux

### macOS

// TODO(yanchith): audit

- Copy `Chromium Embedded Framework.framework` to
  `/Library/Frameworks` (so that it can be found by rustc in compile
  time and the dynamic linker in runtime)

- Copy the dylibs in `Chromium Embedded Framework.framework/Libraries`
  to `/usr/lib` or `/usr/local/lib` (so that they can be found by the
  dynamic linker in runtime)

- Copy the `Chromium Embedded Framework.framework/Resources` directory
  to this directory (so that it can be accessed by the executable)

- Copy `libswiftshader_libEGL.dylib` and
  `libswiftshader_libGLESv2.dylib` from `Chromium Embedded
  Framework.framework/Libraries` to `./target/debug/examples/` in this
  directory (so that the runtime gl implementation loader in chromium
  can find them)
