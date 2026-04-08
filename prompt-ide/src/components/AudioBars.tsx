import { useEffect, useRef, useState } from 'react';

interface AudioBarsProps {
  stream: MediaStream | null;
  barCount?: number;
  color?: string;
  height?: number;
}

export function AudioBars({ stream, barCount = 5, color = 'var(--color-danger)', height = 20 }: AudioBarsProps) {
  const [levels, setLevels] = useState<number[]>(new Array(barCount).fill(0));
  const ctxRef = useRef<AudioContext | null>(null);
  const rafRef = useRef<number>(0);

  useEffect(() => {
    if (!stream) {
      setLevels(new Array(barCount).fill(0));
      return;
    }

    const ctx = new AudioContext();
    const source = ctx.createMediaStreamSource(stream);
    const analyser = ctx.createAnalyser();
    analyser.fftSize = 32;
    analyser.smoothingTimeConstant = 0.6;
    source.connect(analyser);

    const data = new Uint8Array(analyser.frequencyBinCount);

    const tick = () => {
      analyser.getByteFrequencyData(data);
      const step = Math.max(1, Math.floor(data.length / barCount));
      const next = Array.from({ length: barCount }, (_, i) => data[i * step] / 255);
      setLevels(next);
      rafRef.current = requestAnimationFrame(tick);
    };
    rafRef.current = requestAnimationFrame(tick);
    ctxRef.current = ctx;

    return () => {
      cancelAnimationFrame(rafRef.current);
      source.disconnect();
      ctx.close();
    };
  }, [stream, barCount]);

  return (
    <div
      className="flex items-end justify-center"
      style={{ gap: 2, height }}
    >
      {levels.map((l, i) => (
        <div
          key={i}
          style={{
            width: 3,
            borderRadius: 1.5,
            backgroundColor: color,
            height: `${Math.max(3, l * height)}px`,
            transition: 'height 80ms ease-out',
          }}
        />
      ))}
    </div>
  );
}
