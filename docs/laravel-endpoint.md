# Receiving articles on a Laravel site

BlogWriter publishes each article as a JSON `POST` to `{base_url}{api_path}`
(default `/api/articles`) with a bearer token. The site just needs a route that
accepts this payload and stores it.

## Payload

```json
{
  "title":    "10 Laravel Tips",
  "slug":     "10-laravel-tips",
  "excerpt":  "A short summary.",
  "body":     "# 10 Laravel Tips\n\n...markdown...",
  "theme":    "Laravel tips",
  "category": "Laravel tips",
  "image":    "data:image/jpeg;base64,/9j/4AAQ...",
  "status":   "published"
}
```

| Field | Notes |
|---|---|
| `title` / `slug` / `body` | Always present; `body` is Markdown |
| `excerpt` | 1–2 sentence summary, may be empty |
| `theme` | The topic the article was generated for |
| `category` | Blog category to file under; falls back to the theme if unset |
| `image` | A **data URL** (web-optimized 16:9-style JPEG), may be empty — store it directly or decode it to a file (see below) |
| `status` | Currently always `published` |

Headers: `Authorization: Bearer <token>`, `Accept: application/json`.

Respond `2xx`; optionally return `{ "id": 123 }` (or `{ "data": { "id": 123 } }`)
— BlogWriter stores it as the article's `remote_id`. Re-publishing sends the
same `slug`, so treat the slug as the upsert key.

## Minimal Laravel implementation

Migration:

```php
Schema::create('posts', function (Blueprint $t) {
    $t->id();
    $t->string('title');
    $t->string('slug')->unique();
    $t->text('excerpt')->nullable();
    $t->longText('body');
    $t->string('theme')->nullable();
    $t->string('category')->nullable();
    $t->string('status')->default('published');
    $t->string('image_path')->nullable();
    $t->timestamps();
});
```

`routes/api.php` (guard with a token — Sanctum, or a simple shared secret):

```php
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Storage;
use App\Models\Post;

Route::post('/articles', function (Request $req) {
    abort_unless(
        hash_equals(config('services.blogwriter.token'), $req->bearerToken() ?? ''),
        401
    );

    $data = $req->validate([
        'title'    => 'required|string',
        'slug'     => 'required|string',
        'excerpt'  => 'nullable|string',
        'body'     => 'required|string',
        'theme'    => 'nullable|string',
        'category' => 'nullable|string',
        'image'    => 'nullable|string',
        'status'   => 'nullable|string',
    ]);

    // Decode the data URL and store the cover image on disk.
    if (!empty($data['image']) && str_starts_with($data['image'], 'data:image')) {
        $binary = base64_decode(substr($data['image'], strpos($data['image'], ',') + 1));
        $path = 'blog/' . $data['slug'] . '.jpg';
        Storage::disk('public')->put($path, $binary);
        $data['image_path'] = $path;
    }
    unset($data['image']);

    $post = Post::updateOrCreate(['slug' => $data['slug']], $data);

    return response()->json(['id' => $post->id], 201);
});
```

`config/services.php`:

```php
'blogwriter' => ['token' => env('BLOGWRITER_TOKEN')],
```

Set the same token in BlogWriter's site form ("Bearer token"). Render `body`
(Markdown) with your preferred parser (e.g. `league/commonmark`).

## Tips

- **Use HTTPS and the canonical host** in the site's Base URL — redirect
  chains (http→https, `www.`) work but add latency and failure modes.
- **Allow large payloads**: the `image` data URL can be several hundred KB.
  If you validate with `max:`, size it generously or drop the limit.
- **Idempotency**: `updateOrCreate` on `slug` means re-publishing an edited
  article updates the existing post instead of duplicating it.
- Errors you return (status + body) are stored verbatim on the article in
  BlogWriter — helpful messages make debugging easier. See
  [troubleshooting.md](troubleshooting.md) for the client-side view.
