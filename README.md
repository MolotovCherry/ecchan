# Ecchan for Linux (for MSI laptops)

Ecchan is a Rust-based ecram library for Rust. It allows you to control a lot of hardware features for your MSI laptop:

- get fan count
- get wmi version
- get fw info (version, date time)
- control shift mode
- control battery charge mode
- enable/disable super battery mode
- get fan 1-4 rpms
- control fan mode
- enable/disable webcam
- enable/disable cooler boost
- swap fn/win keys
- enable/disable mic/audio mute leds
- view real time cpu/gpu temp/fan speed
- view/set cpu/gpu fan/temp/hysteresis curves
- get raw/pretty ec dump
- custom model specific functions (things like overdrive, usb power share, etc)

Currently only supports WMI2 models.

# Using

This currently requires the Linux `ec_sys` driver is loaded with `write_support` enabled, however compatibility with [msi-ec](https://github.com/BeardOverflow/msi-ec) driver is planned, and using the driver will be preferred over direct ecram editing. Please note that msi-ec does not support editing of fan curves or model specific functions.

# Adding support for your model

1. check if your ec version is supported under the `fw` folder.
2. if it isn't, then check [msi-ec.c](https://github.com/BeardOverflow/msi-ec/blob/main/msi-ec.c) for your specific version
3. ff it exists there, you may create a new module under `wmi2` for your ec version. The format is similar, so it should be relatively easy to copy it from the c file.
4. if it does not exist, please check [here](https://github.com/BeardOverflow/msi-ec/blob/main/docs/device_support_guide.md) for how to get the values for your specific model.

You may add support for model specific features that are unavailable under fw in the `models` directory. You may find the the model string for your laptop by doing a `cat /sys/class/dmi/id/product_name`.

# Long term goals

Might think about upgrading this to a kernel driver in the far future. But this will have to be when most kernels enable Rust support by default.

# AI Policy

This project has a hard ban on AI usage.

No AI was used in the making of this library (whether by programming or using it to understand things). It is entirely human produced.

# Contributions

Contributions to further improve and expand ec version and model support are welcome. As per the AI policy, AI is not allowed to be used in the making of your contribution, and any such PRs and issues will immediately be rejected. If you used AI, please do not contribute.

# Disclaimer

This software is a hobby project. Every best effort has been made to ensure everything works properly, however it is still technically possible - even though highly unlikely - that something goes wrong. An extensive test suite was made precisely to ensure safety. If you are wary of a product that edits your ecram, you are encouraged to read, understand, and verify the memory addresses and values for your particular ec version and computer model.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
