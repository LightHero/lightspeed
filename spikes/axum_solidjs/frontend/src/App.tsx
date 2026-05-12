import { createSignal } from 'solid-js';
import createClient from 'openapi-fetch';
import type { paths } from './api/schema';

// `createClient<paths>` produces a fetch wrapper whose `GET`, `POST`, ...
// methods are constrained to the URL templates declared in the generated
// `paths` type. The request body and response type for each endpoint are
// inferred from the OpenAPI document.
const api = createClient<paths>({ baseUrl: '' });

function App() {
  const [input, setInput] = createSignal('hello from solid');
  const [echo, setEcho] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(false);

  const onSubmit = async (e: SubmitEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      // Type-checked end-to-end: the path "/api/echo", the request body shape,
      // and the type of `data.echo` are all derived from the generated schema.
      const { data } = await api.POST('/api/echo', {
        body: { message: input() },
      });
      if (!data) {
        throw new Error('request failed');
      }
      setEcho(data.echo);
    } catch (err) {
      setEcho(null);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <main class="container">
      <article>
        <header>
          <hgroup>
            <h1>Echo spike</h1>
            <p>Axum backend, SolidJS + TS frontend via Vite, types generated from OpenAPI.</p>
          </hgroup>
        </header>

        <form onSubmit={onSubmit}>
          <label>
            Message
            <input
              type="text"
              value={input()}
              onInput={(e) => setInput(e.currentTarget.value)}
              disabled={loading()}
              placeholder="Type something and submit"
              autofocus
            />
          </label>
          <button type="submit" aria-busy={loading()}>
            {loading() ? 'Sending…' : 'Send to /api/echo'}
          </button>
        </form>

        {echo() !== null && (
          <article>
            <strong>Server echoed:</strong> <code>{echo()}</code>
          </article>
        )}

        {error() !== null && (
          <article style="border-color: var(--pico-del-color, #c33);">
            <strong>Error:</strong> {error()}
          </article>
        )}

        <footer>
          <small>
            <a href="/api/docs" target="_blank" rel="noopener">
              API docs (Scalar)
            </a>{' '}
            ·{' '}
            <a href="/api/openapi.json" target="_blank" rel="noopener">
              openapi.json
            </a>
          </small>
        </footer>
      </article>
    </main>
  );
}

export default App;
