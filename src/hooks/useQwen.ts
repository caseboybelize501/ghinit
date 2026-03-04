import { useState, useCallback, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { locateQwen, qwenGenerate } from '../lib/api';
import type { QwenLocation } from '../types';

export function useQwen() {
  const [location, setLocation] = useState<QwenLocation | null>(null);
  const [scanning, setScanning] = useState(false);
  const [generating, setGenerating] = useState(false);
  const [streamedText, setStreamedText] = useState('');
  const unlistenRef = useRef<(() => void) | null>(null);

  const scan = useCallback(async () => {
    setScanning(true);
    try {
      const loc = await locateQwen();
      setLocation(loc);
      return loc;
    } catch (e) {
      console.error('Qwen scan failed:', e);
      return null;
    } finally {
      setScanning(false);
    }
  }, []);

  // Auto-scan on mount
  useEffect(() => { scan(); }, [scan]);

  const generate = useCallback(async (
    prompt: string,
    system?: string,
    onToken?: (token: string) => void,
    projectPath?: string | null,
  ): Promise<string> => {
    if (!location?.found) throw new Error('Qwen not found');
    setStreamedText('');
    setGenerating(true);

    // Set up token streaming listener
    const unlisten = await listen<string>('qwen-token', (event) => {
      setStreamedText(prev => prev + event.payload);
      onToken?.(event.payload);
    });
    unlistenRef.current = unlisten;

    try {
      const result = await qwenGenerate(location, prompt, system, projectPath);
      return result;
    } finally {
      unlisten();
      unlistenRef.current = null;
      setGenerating(false);
    }
  }, [location]);

  return { location, scanning, generating, streamedText, scan, generate };
}
