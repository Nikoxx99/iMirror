export function Architecture() {
  return (
    <section className="bg-ink px-6 py-20 text-white">
      <div className="mx-auto max-w-6xl">
        <h2 className="text-3xl font-semibold">Video stays on your local network</h2>
        <div className="mt-8 grid gap-4 md:grid-cols-3">
          {["Safari or Chrome camera", "Encrypted local WebRTC", "iMirror Camera on Windows"].map((item) => (
            <div key={item} className="rounded-xl border border-white/10 bg-white/[0.03] p-5">
              <p className="font-medium">{item}</p>
              <p className="mt-2 text-sm leading-6 text-slate-400">
                No account, remote relay or automatic recording is part of the default path.
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
