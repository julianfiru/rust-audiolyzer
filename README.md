# Audiolyzer Pro

A high-performance, real-time audio spectrum analyzer and oscilloscope for the terminal, written entirely in Rust. This application utilizes a Terminal User Interface (TUI) to visualize system audio output with extreme precision and minimal overhead.

## Features

- **Real-Time Spectrum Analysis**: Visualizes frequency domain data using Fast Fourier Transform (FFT) across ISO standard frequency bands (20Hz to 20kHz).
- **Sub-Cell Precision Rendering**: Employs 8-level sub-cell vertical resolution to render spectrum bars smoothly within the terminal constraints.
- **Time-Domain Oscilloscope**: Displays raw PCM audio signal waveforms in real-time.
- **Stereo VU Meter**: Accurate dual-channel (Left/Right) RMS and Peak level metering with clipping detection.
- **Lock-Free Concurrency**: Uses Single-Producer Single-Consumer (SPSC) ring buffers to decouple the high-frequency audio capture thread from the UI rendering loop, guaranteeing zero audio dropouts.
- **Zero-Allocation Hot Loop**: DSP calculations (Windowing, FFT, Binning, Ballistics) operate on pre-allocated buffers, ensuring strict 60 FPS rendering without garbage collection pauses or heap allocations.
- **Dynamic Configuration**: Switch between Hann, Hamming, and Rectangular windowing functions, or toggle between Logarithmic and Linear frequency scales on the fly.

## Technical Architecture

- **Audio Capture**: Utilizes `cpal` for cross-platform audio I/O. On Windows, it leverages WASAPI Loopback to capture internal desktop audio directly from the sound card.
- **DSP Engine**: Powered by `rustfft` for highly optimized Fourier transformations. Includes custom implementations for exponential decay ballistics and peak falloff dynamics.
- **TUI Renderer**: Built with `ratatui` and `crossterm` for a responsive, interactive terminal interface that supports resizing and theme switching.

## Installation

Ensure you have the Rust toolchain installed (version 1.70 or higher is recommended).

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/audiolyzer.git
   cd audiolyzer
   ```

2. Build and run the application:
   ```bash
   cargo run --release
   ```

## Controls and Keybindings

The application provides an interactive dashboard with the following controls:

- `1` : Switch to Spectrum Analyzer Mode
- `2` : Switch to Waveform Oscilloscope Mode
- `3` : Switch to Stereo VU Meter Mode
- `Tab` : Cycle through color themes (Cyberpunk, Matrix, Fire, Studio Dark)
- `W` : Toggle Windowing Function (Hann / Rectangular / Hamming)
- `S` : Toggle Frequency Scale (Logarithmic / Linear)
- `Up` / `Down` : Adjust gain sensitivity (+3dB / -3dB)
- `Space` : Freeze/Pause the visualizer
- `H` : Open the Help Modal
- `Q` or `Esc` : Quit the application safely

## Troubleshooting

- **No audio detected (Zero bars)**: Ensure that you have active audio playing on your system (e.g., music, video). 
- **Windows Microphone Privacy**: Windows may block terminal applications from accessing the audio loopback device. Go to Settings -> Privacy & Security -> Microphone and ensure "Let desktop apps access your microphone" is enabled.

## License

This project is licensed under the MIT License. See the LICENSE file for details.
