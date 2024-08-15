# Lightarti-rest - ARCHIVED

This project has been archived by the [C4DT](https://c4dt.epfl.ch) Factory team.
It is kept here for reference and for old URLs.
You can find more information in our [showcase](https://factory.c4dt.org/showcase/lightarti/presentation)
and find our contact information.

## Description

lightarti-rest is a simple wrapper around [arti](https://gitlab.torproject.org/tpo/core/arti) that leverages arti to enable mobile apps to easily make anonymous REST requests via the Tor network. lightarti-rest makes two changes on top of arti. First, lightarti-rest provides a simple wrapper for REST requests, to make calling HTTP(S) API endpoints easier. Second, lightarti-rest provides the option to use a customized, and therefore potentially much smaller, consensus.

> :warning: **Warning: lightarti-rest is not secure in all situations** lightarti-rest modifies several core parts of `arti`. These modifications result in lightarti-rest not providing the same guarantees as arti and the stock Tor client. You will have to verify on your own whether these weakened guarantees are acceptable in your situation. See the reliability section below to check what aspects of your system you need to consider to decide whether lightarti-rest is secure for you.

lightarti-rest is written in Rust and can therefore be compiled into native libraries that are easy to integrate into mobile applications. All credits for enabling this approach go to the authors of [arti](https://gitlab.torproject.org/tpo/core/arti). Using a native library ensures that lightarti-rest can be bundled with Android and iOS applications, rather than relying on external Tor proxies that must be installed separately.

To facilitate integration in mobile applications, we provide the [lightarti-rest-ios](https://github.com/c4dt/lightarti-rest-ios) and [lightarti-rest-android](https://github.com/c4dt/lightarti-rest-android) libraries. The following graph shows how these libraries interact.

```
lightarti-rest-ios   lightarti-rest-android
         I                      I
              lightarti-rest
                    I
                  arti
```

`lightarti-rest` provides a synchronous interface to send REST requests over Tor, and
to receive the answers. It also does the TLS handling of the connection. The
asynchronous part needs to be done by the Android and iOS library.

## Reliability

Even though we tried our best to produce good code, this should not be used in a
mission critical piece of software without proper consideration of the following:

- arti itself mentions it's not ready for prime-time since it does not implement all protections that the stock Tor client does. See their [README](https://gitlab.torproject.org/tpo/core/arti/-/blob/main/README.md) to check whether your application can be secure without those protections.
- lightarti-rest uses a customized consensus to reduce bandwidth cost for infrequent use of lightarti-rest. As a result, the consensus is not signed by the default Tor directory authorities, but instead by the creator of the app. This approach in general is not secure. However, since the app developer already determines which code is loaded (and could therefore disable consensus validation altogether, or deanonymize users directly), it is likely acceptable in the case of mobile applications that directly include lightarti-rest.

In summary, you should only use `lightarti-rest` when you understand the differences between `arti` and `lightarti-rest` and Tor, and you are certain that the loss of protection with respect to Tor does not harm the security of your users.

## Mobile Libraries

Because iOS XCFrameworks and libraries need to reside in a separate GitHub repository,
we decided to have both Android and iOS in a separate repository:

- [Android](https://github.com/c4dt/lightarti-rest-android)
- [iOS](https://github.com/c4dt/lightarti-rest-ios)

You can have a look at these repositories for further instructions how to use the
library.

### Test apps

The Android test app is in the `lightarti-rest-android` repository.
For the iOS test app, see here:
[lightarti-rest-ios-test](https://github.com/c4dt/lightarti-rest-ios-test)

## Roadmap:

- v0.1 - Simple GET request and reply using TLS - done
- v0.2 - Optimized setting up of the Tor circuit - first version done
- v0.3 - Add certificate configurations to the tor-cache directory
- v0.4 - Re-use existing tor connection for multiple requests
- v0.5 - Configure lightarti-rest to either use standard tor, or a pre-configured circuit
- v1.0 - Once arti is deemed stable enough, and other people have looked and used this code

### Versioning

The following repos follow the same versioning related to the major and minor version:

- lightarti-rest
- lightarti-rest-android
- lightarti-rest-ios

The patch version of the three will differ and reflect internal updates, with the
following constraints:

- `patch_version(lightarti_rest_android) >= patch_version(lightarti_rest)`
- `patch_version(lightarti_rest_ios) >= patch_version(lightarti_rest)`
- a new lightarti-rest patch version must be bigger than both the current lightarti-rest-android and
  lightarti-rest-ios patch version

This allows us to quickly verify that a given library version has at least some patches
from the lightarti-rest code by simply looking at the version number.

### Releasing a new version

To release a new version, simply add a new tag to the repo and push it:

```
git tag 0.4.4
git push --tag
```

The github-workflow will then create a new release.
Please make sure that the version fits the above constraints.

## Directories

- `./` is the rust library for the wrapper with arti
- `./ios` holds scripts to create the `XCFramework` used in the `lightarti-rest-ios`
- `./tools` holds scripts to generate the files for the offline setup of the tor circuits

# Custom Tor Consensus

Tor relies on directory information to describe the state of the network. This information is updated every hour and signed by the Tor directory authorities. Tor clients, including arti, aim to always retrieve and use the latest Tor directory information. Updating this information requires bandwidth. When using Tor for anonymous browsing, this overhead is small. When using Tor for infrequent and small anonymous requests, however, the overhead of updating the directory information quickly starts to dominate.

To reduce the bandwidth overhead of downloading the full Tor directory, lightarti-rest instead relies on a subset of all the Tor directory information. This subset can then consist of a smaller set of reliable nodes. This modified directory can be used for a long time (as much as up to a week). Lightarti-rest achieves this in the following way:

1. It provides [scripts for generating and signing custom Tor directory information](tools/README.md). Apps have to download these files over a non-tor connection and store them for subsequent calls to lightarti-rest.
2. This script can also be used to compute a tiny churn file of no-longer-available Tor nodes for a given custom Tor directory. This ensures lightarti-rest can still quickly build circuits even with an older custom directory.
3. It uses a [modified arti directory manager](src/lightarti/tor-dirmgr) that parses, verifies and uses the cached and custom consensus files and applies corrections described by the churn file.

The custom directory information, with the exception of the churn file, is signed. Thus making it harder to provide incorrect directory information as operators cannot later repudiate having done so. The churn file itself is not signed. A malicious operator could therefore try to disable all but a few corrupted Tor nodes. To prevent this attack, lightarti-rest limits the size of the churn file to 1/6th of the nodes. Thus ensuring that an attacker can at most disable half of the nodes for each position.

As discussed above, the use of custom directory information might not be secure in your deployment scenario. It is essential that you perform your own analysis to determine whether using Lightarti-rest is secure for you.

# Testing the library

The library has some simple tests that contact some servers using lightarti-rest.
Before running the test, the cache information needs to be updated.
So a full test can be started by running the following commands:

```bash
make dircache
cargo test
```

# Notes on building the library for x86_64 devices

To build this library for a x86_64 device, Android NDK version 25.2.9519653 must be installed and
the installation location must be pointed to in the `ANDROID_NDK_HOME` environment variable.

# License

The code is licensed under the MIT license.

# Contributors

`lightarti-rest` is maintained by the [Center for Digital Trust](https://c4dt.org/). The following people contributed to the implementation of `lightarti-rest`:

- Linus Gasser, C4DT
- Valérian Rousset, C4DT
- Christian Grigis, C4DT
- Carine Dengler, C4DT
- Laurent Girod, SPRING Lab, EPFL

Analysis and design by:

- Wouter Lueks, SPRING Lab, EPFL
- Carmela Troncoso, SPRING Lab, EPFL

External contributors:
- Benjamin Erhart, https://github.com/tladesignz
