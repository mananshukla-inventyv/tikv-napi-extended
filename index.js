/* tslint:disable */
/* eslint-disable */
/* prettier-ignore */

/* auto-generated by NAPI-RS */

const { existsSync, readFileSync } = require('fs')
const { join } = require('path')

const { platform, arch } = process

let nativeBinding = null
let localFileExisted = false
let loadError = null

function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      const lddPath = require('child_process').execSync('which ldd').toString().trim()
      return readFileSync(lddPath, 'utf8').includes('musl')
    } catch (e) {
      return true
    }
  } else {
    const { glibcVersionRuntime } = process.report.getReport().header
    return !glibcVersionRuntime
  }
}

switch (platform) {
  case 'android':
    switch (arch) {
      case 'arm64':
        localFileExisted = existsSync(join(__dirname, 'tikv-napi-extended.android-arm64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./tikv-napi-extended.android-arm64.node')
          } else {
            nativeBinding = require('tikv-napi-extended-android-arm64')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm':
        localFileExisted = existsSync(join(__dirname, 'tikv-napi-extended.android-arm-eabi.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./tikv-napi-extended.android-arm-eabi.node')
          } else {
            nativeBinding = require('tikv-napi-extended-android-arm-eabi')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Android ${arch}`)
    }
    break
  case 'win32':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(
          join(__dirname, 'tikv-napi-extended.win32-x64-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./tikv-napi-extended.win32-x64-msvc.node')
          } else {
            nativeBinding = require('tikv-napi-extended-win32-x64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'ia32':
        localFileExisted = existsSync(
          join(__dirname, 'tikv-napi-extended.win32-ia32-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./tikv-napi-extended.win32-ia32-msvc.node')
          } else {
            nativeBinding = require('tikv-napi-extended-win32-ia32-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'tikv-napi-extended.win32-arm64-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./tikv-napi-extended.win32-arm64-msvc.node')
          } else {
            nativeBinding = require('tikv-napi-extended-win32-arm64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`)
    }
    break
  case 'darwin':
    localFileExisted = existsSync(join(__dirname, 'tikv-napi-extended.darwin-universal.node'))
    try {
      if (localFileExisted) {
        nativeBinding = require('./tikv-napi-extended.darwin-universal.node')
      } else {
        nativeBinding = require('tikv-napi-extended-darwin-universal')
      }
      break
    } catch {}
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(join(__dirname, 'tikv-napi-extended.darwin-x64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./tikv-napi-extended.darwin-x64.node')
          } else {
            nativeBinding = require('tikv-napi-extended-darwin-x64')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'tikv-napi-extended.darwin-arm64.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./tikv-napi-extended.darwin-arm64.node')
          } else {
            nativeBinding = require('tikv-napi-extended-darwin-arm64')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`)
    }
    break
  case 'freebsd':
    if (arch !== 'x64') {
      throw new Error(`Unsupported architecture on FreeBSD: ${arch}`)
    }
    localFileExisted = existsSync(join(__dirname, 'tikv-napi-extended.freebsd-x64.node'))
    try {
      if (localFileExisted) {
        nativeBinding = require('./tikv-napi-extended.freebsd-x64.node')
      } else {
        nativeBinding = require('tikv-napi-extended-freebsd-x64')
      }
    } catch (e) {
      loadError = e
    }
    break
  case 'linux':
    switch (arch) {
      case 'x64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'tikv-napi-extended.linux-x64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./tikv-napi-extended.linux-x64-musl.node')
            } else {
              nativeBinding = require('tikv-napi-extended-linux-x64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'tikv-napi-extended.linux-x64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./tikv-napi-extended.linux-x64-gnu.node')
            } else {
              nativeBinding = require('tikv-napi-extended-linux-x64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'arm64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'tikv-napi-extended.linux-arm64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./tikv-napi-extended.linux-arm64-musl.node')
            } else {
              nativeBinding = require('tikv-napi-extended-linux-arm64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'tikv-napi-extended.linux-arm64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./tikv-napi-extended.linux-arm64-gnu.node')
            } else {
              nativeBinding = require('tikv-napi-extended-linux-arm64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'arm':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'tikv-napi-extended.linux-arm-musleabihf.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./tikv-napi-extended.linux-arm-musleabihf.node')
            } else {
              nativeBinding = require('tikv-napi-extended-linux-arm-musleabihf')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'tikv-napi-extended.linux-arm-gnueabihf.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./tikv-napi-extended.linux-arm-gnueabihf.node')
            } else {
              nativeBinding = require('tikv-napi-extended-linux-arm-gnueabihf')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'riscv64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'tikv-napi-extended.linux-riscv64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./tikv-napi-extended.linux-riscv64-musl.node')
            } else {
              nativeBinding = require('tikv-napi-extended-linux-riscv64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'tikv-napi-extended.linux-riscv64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./tikv-napi-extended.linux-riscv64-gnu.node')
            } else {
              nativeBinding = require('tikv-napi-extended-linux-riscv64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 's390x':
        localFileExisted = existsSync(
          join(__dirname, 'tikv-napi-extended.linux-s390x-gnu.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./tikv-napi-extended.linux-s390x-gnu.node')
          } else {
            nativeBinding = require('tikv-napi-extended-linux-s390x-gnu')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`)
    }
    break
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`)
}

if (!nativeBinding) {
  if (loadError) {
    throw loadError
  }
  throw new Error(`Failed to load native binding`)
}

const { getNextKey, initClient, startLogger, getDocument, addDocument, replaceDocument, getBatchUsingScan, getBatch, getDocumentsV2, deleteDocument, getNextCounter } = nativeBinding

module.exports.getNextKey = getNextKey
module.exports.initClient = initClient
module.exports.startLogger = startLogger
module.exports.getDocument = getDocument
module.exports.addDocument = addDocument
module.exports.replaceDocument = replaceDocument
module.exports.getBatchUsingScan = getBatchUsingScan
module.exports.getBatch = getBatch
module.exports.getDocumentsV2 = getDocumentsV2
module.exports.deleteDocument = deleteDocument
module.exports.getNextCounter = getNextCounter
