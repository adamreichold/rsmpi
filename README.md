# Message Passing Interface bindings for Rust

[![Travis build status][travis-shield]][travis] [![License: MIT][license-shield]][license]

The [Message Passing Interface][MPI] (MPI) is a specification for a
message-passing style concurrency library. Implementations of MPI are often used to structure
parallel computation on High Performance Computing systems. The MPI specification describes
bindings for the C programming language (and through it C++) as well as for the Fortran
programming language. This library tries to bridge the gap into a more rustic world.

[license-shield]: https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square
[license]: https://github.com/bsteinb/rsmpi/LICENSE
[travis-shield]: https://img.shields.io/travis/bsteinb/rsmpi.svg?style=flat-square
[travis]: https://travis-ci.org/bsteinb/rsmpi
[MPI]: http://www.mpi-forum.org

## Requirements

An implementation of the C language interface of MPI. These bindings have been tested with:

- [OpenMPI][OpenMPI] 1.8.6
- [MPICH][MPICH] 3.1.4

[OpenMPI]: https://www.open-mpi.org
[MPICH]: https://www.mpich.org

## Building

```
cargo build
```

Uses `rust-bindgen` to generate FFI definitions, see the [bindgen project page][bindgen] for troubleshooting.

[bindgen]: https://github.com/crabtw/rust-bindgen

## Usage

Add the `mpi` crate as a dependency in your `Cargo.toml`:

```
[dependencies]
mpi = "0.1.0"
```

Then use it in your program like this:

```
extern crate mpi;

use mpi::traits::*;

fn main() {
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let size = world.size();
    let rank = world.rank();

    if size != 2 {
        panic!("Size of MPI_COMM_WORLD must be 2, but is {}!", size);
     }

    match rank {
        0 => {
            let msg = vec![4.0f64, 8.0, 15.0];
            world.process_at_rank(rank + 1).send(&msg[..]);
        }
        1 => {
            let (msg, status) = world.receive_vec::<f64>();
            println!("Process {} got message {:?}.\nStatus is: {:?}", rank, msg, status);
        }
        _ => unreachable!()
    }
}
```

## Features

The bindings follow the MPI 3.1 specification.

Currently supported:

- **Groups, Contexts, Communicators**: Only rudimentary features are supported so far.
- **Point to point communication**: Most of the blocking, standard mode functions are supported.
Blocking communication in buffered, synchronous and ready mode are not yet supported. Neither
are the non-blocking functions.
- **Collective communication**: Blocking barrier, broadcast and gather operations.
- **Datatypes**: Bridging between Rust types and MPI basic types as well as custom MPI datatypes
which can act as views into buffers.

Not supported (yet):

- Process management
- One-sided communication (RMA)
- MPI parallel I/O
- A million small things

## Documentation

```
cargo doc
```

Hosted documentation coming soon.

## Examples

See files in `examples/`.

## License

The MIT license, see the file `LICENSE`.