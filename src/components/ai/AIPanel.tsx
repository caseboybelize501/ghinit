import { useState, useRef, useEffect } from 'react';
import { useQwen } from '../../hooks/useQwen';
import type { QwenLocation } from '../../types';

interface Props {
  qwenLocation: QwenLocation | null;
}

interface Message {
  role: 'user' | 'assistant';
  content: string;
  streaming?: boolean;
}

const SYSTEM = `You are Qwen Coder, an expert programming assistant embedded in PurposeForge, 
a modular software builder. Help users with code, architecture, debugging, and project planning.
Be concise but thorough. Use markdown for code blocks.`;

export default function AIPanel({ qwenLocation }: Props) {
  const [messages, setMessages] = useState<Message[]>([
    { role: 'assistant', content: 'Hi! I\'m Qwen Coder. Ask me anything about your code, architecture, or project design. I can also help you refine what the Builder should generate.' }
  ]);
  const [input, setInput] = useState('');
  const { generate, generating } = useQwen();
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const sendMessage = async () => {
    if (!input.trim() || generating) return;
    if (!qwenLocation?.found) return;

    const userMsg = input.trim();
    setInput('');

    setMessages(prev => [
      ...prev,
      { role: 'user', content: userMsg },
      { role: 'assistant', content: '', streaming: true },
    ]);

    try {
      let streamed = '';
      await generate(userMsg, SYSTEM, (token) => {
        streamed += token;
        setMessages(prev => prev.map((m, i) =>
          i === prev.length - 1 ? { ...m, content: streamed } : m
        ));
      });

      // Mark done
      setMessages(prev => prev.map((m, i) =>
        i === prev.length - 1 ? { ...m, streaming: false } : m
      ));
    } catch (e: any) {
      setMessages(prev => prev.map((m, i) =>
        i === prev.length - 1 ? { ...m, content: `Error: ${e}`, streaming: false } : m
      ));
    }
  };

  const handleKey = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  return (
    <div className="panel ai-panel">
      <div className="panel-header">
        <h1>AI Chat</h1>
        {qwenLocation?.found
          ? <span className="status-chip green">🟢 {qwenLocation.model ?? 'Qwen'}</span>
          : <span className="status-chip red">🔴 Qwen not connected</span>
        }
      </div>

      {!qwenLocation?.found && (
        <div className="warning-box">
          Qwen Coder is not running. Install via Ollama:<br />
          <code>ollama pull qwen2.5-coder</code><br />
          Then click <strong>Rescan</strong> in the sidebar.
        </div>
      )}

      <div className="chat-messages">
        {messages.map((msg, i) => (
          <div key={i} className={`chat-message ${msg.role}`}>
            <div className="chat-avatar">{msg.role === 'user' ? '🧑' : '🤖'}</div>
            <div className="chat-bubble">
              <pre className="chat-content">{msg.content || (msg.streaming ? '▌' : '')}</pre>
            </div>
          </div>
        ))}
        <div ref={bottomRef} />
      </div>

      <div className="chat-input-area">
        <textarea
          className="chat-input"
          rows={3}
          placeholder={qwenLocation?.found ? 'Ask Qwen anything... (Shift+Enter for newline)' : 'Connect Qwen to start chatting'}
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyDown={handleKey}
          disabled={!qwenLocation?.found || generating}
        />
        <button
          className="btn btn-primary send-btn"
          onClick={sendMessage}
          disabled={!qwenLocation?.found || generating || !input.trim()}
        >
          {generating ? '⏳' : '→'}
        </button>
      </div>
    </div>
  );
}
