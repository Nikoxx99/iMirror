import { Download, Github } from "lucide-react";

export function Hero() {
  return (
    <section className="min-h-screen bg-ink px-6 py-8 text-white">
      <div className="mx-auto flex min-h-[calc(100vh-4rem)] max-w-6xl flex-col justify-center">
        <img
          src={`${import.meta.env.BASE_URL}imirror-icon.png`}
          alt="iMirror phone-to-webcam icon"
          className="mb-8 h-20 w-20 object-contain"
        />
        <p className="text-sm font-medium text-brand">iMirror</p>
        <h1 className="mt-3 max-w-4xl text-5xl font-semibold tracking-tight sm:text-7xl">
          Use your phone as a Windows webcam.
        </h1>
        <p className="mt-6 max-w-2xl text-lg leading-8 text-slate-300">
          Turn an iPhone or Android camera into a private, low-latency virtual webcam for Windows 11. Pair with a QR
          code, stream over local Wi-Fi, and select iMirror Camera in meetings or OBS.
        </p>
        <div className="mt-8 flex flex-wrap gap-3">
          <a
            href="https://github.com/Nikoxx99/iMirror/releases/latest"
            className="inline-flex items-center gap-2 rounded-lg bg-brand px-5 py-3 font-semibold text-ink"
          >
            <Download className="h-5 w-5" />
            Download for Windows 11
          </a>
          <a
            href="https://github.com/Nikoxx99/iMirror"
            className="inline-flex items-center gap-2 rounded-lg border border-white/20 px-5 py-3 font-semibold text-white"
          >
            <Github className="h-5 w-5" />
            View source
          </a>
        </div>
      </div>
    </section>
  );
}
