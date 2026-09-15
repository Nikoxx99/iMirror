const features = [
  "iPhone and Android browser capture — no mobile app",
  "Local WebRTC transport with short-lived QR pairing",
  "iMirror Camera output for Windows apps",
  "No accounts, telemetry, subscriptions or cloud video",
  "720p balanced and low-latency quality modes",
  "Open-source Rust, Tauri and TypeScript"
];

export function Features() {
  return (
    <section className="bg-slate-950 px-6 py-20 text-white">
      <div className="mx-auto max-w-6xl">
        <h2 className="text-3xl font-semibold">A DroidCam alternative you can inspect.</h2>
        <div className="mt-8 grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {features.map((feature) => (
            <div key={feature} className="rounded-xl border border-white/10 bg-white/[0.03] p-5 text-sm text-slate-300">
              {feature}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
