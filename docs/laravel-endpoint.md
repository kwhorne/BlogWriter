# Receiving articles on a Laravel site

BlogWriter publishes each article as a JSON `POST` to `{base_url}{api_path}`
(default `/api/articles`) with a bearer token. The site just needs a route that
accepts this payload and stores it.

## Payload

```json
{
  "title":   "10 Laravel Tips",
  "slug":    "10-laravel-tips",
  "excerpt": "A short summary.",
  "body":    "# 10 Laravel Tips\n\n...markdown...",
  "theme":   "Laravel tips",
  "image":   "data:image/jpeg;base64,/9j/4AAQ...",  // 16:9 cover, may be empty
  "status":  "published"
}
```

`image` is a **data URL** (a web-optimized JPEG, may be empty). Store it directly
or decode it to a file — see below.

Headers: `Authorization: Bearer <token>`, `Accept: application/json`.
Respond `2xx`; optionally return `{ "id": 123 }` (or `{ "data": { "id": 123 } }`)
— BlogWriter stores it as the article's `remote_id`.

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
    $t->string('status')->default('published');
    $t->string('image_path')->nullable();
    $t->timestamps();
});
```

`routes/api.php` (guard with a token — Sanctum, or a simple shared secret):

```php
use Illuminate\Http\Request;
use App\Models\Post;

Route::post('/articles', function (Request $req) {
    abort_unless(
        hash_equals(config('services.blogwriter.token'), $req->bearerToken() ?? ''),
        401
    );

    $data = $req->validate([
        'title'   => 'required|string',
        'slug'    => 'required|string',
        'excerpt' => 'nullable|string',
        'body'    => 'required|string',
        'theme'   => 'nullable|string',
        'image'   => 'nullable|string',
        'status'  => 'nullable|string',
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
