use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div { class: "app-container",
            Main {}
        }
    }
}

#[component]
pub fn Main() -> Element {
    let mut show_post_form = use_signal(|| false);
    let mut post_title = use_signal(|| String::new());
    let mut post_body = use_signal(|| String::new());
    let mut posts = use_signal(Vec::<Post>::new);

    // Load posts on component mount
    use_future(move || async move {
        if let Ok(loaded_posts) = get_posts().await {
            *posts.write() = loaded_posts;
        }
    });

    rsx! {
        MenuComponent { show_post_form }

        main { class: "main-content",
            if show_post_form() {
                PostEntryComponent {
                    post_title,
                    post_body,
                    on_post_created: move || {
                        // Reset form and show posts after creation
                        post_title.set(String::new());
                        post_body.set(String::new());
                        show_post_form.set(false);

                        // Reload posts
                        spawn(async move {
                            if let Ok(loaded_posts) = get_posts().await {
                                *posts.write() = loaded_posts;
                            }
                        });
                    }
                }
            } else {
                PostsComponent { posts }
            }
        }
    }
}

#[component]
fn MenuComponent(show_post_form: Signal<bool>) -> Element {
    rsx! {
        div { class: "menu-container",
            button {
                onclick: move |_| show_post_form.set(!show_post_form()),
                if show_post_form() { "← Back" } else { "+ Create Post" }
            }
            ul {
                li { "Home" }
                li { "Popular" }
                li { "Categories" }
                li { "About me" }
            }
        }
    }
}

#[component]
fn PostsComponent(posts: Signal<Vec<Post>>) -> Element {
    rsx! {
        div { class: "main-inner-container",
            if posts().is_empty() {
                div { class: "empty-state", "No posts yet. Create the first one!" }
            } else {
                for post in posts.iter() {
                    PostComponent { key: "{post.id}", post: post.clone() }
                }
            }
        }
    }
}

#[component]
fn PostComponent(post: Post) -> Element {
    rsx! {
        article { class: "post-container",
            header { class: "post-header",
                h1 { {post.title} }
            }
            div { class: "post-content",
                div { class: "post-body", {post.post_body} }
            }
            footer { class: "post-footer",
                span { "Shreyas" }
                time { {post.created_time} }
            }
        }
    }
}

#[component]
fn PostEntryComponent(
    post_title: Signal<String>,
    post_body: Signal<String>,
    on_post_created: EventHandler<()>,
) -> Element {
    let mut is_posting = use_signal(|| false);

    rsx! {
        div { class: "main-inner-container",
            div { class: "input-container",
                input {
                    r#type: "text",
                    placeholder: "Post Title",
                    value: "{post_title}",
                    oninput: move |e| post_title.set(e.value())
                }
            }
            div { class: "input-container",
                textarea {
                    class: "textarea",
                    placeholder: "Post Body",
                    value: "{post_body}",
                    oninput: move |e| post_body.set(e.value())
                }
            }
            div { class: "actions",
                button {
                    disabled: is_posting(),
                    onclick: move |_| {
                        is_posting.set(true);
                        spawn({
                            let post_title = post_title.clone();
                            let post_body = post_body.clone();
                            let on_post_created = on_post_created.clone();

                            async move {
                                let result = create_post(post_title(), post_body()).await;
                                is_posting.set(false);

                                if result.is_ok() {
                                    on_post_created.call(());
                                }
                            }
                        });
                    },
                    if is_posting() { "Posting..." } else { "Post" }
                }
            }
        }
    }
}

#[server]
async fn create_post(post_title: String, post_body: String) -> Result<(), ServerFnError> {
    let conn = rusqlite::Connection::open("./data.db3")?;

    //Uncomment the below lines if running for the first time to create the posts table

    //// Create posts table if it doesn't exist
    // conn.execute(
    //     r#"
    //     CREATE TABLE IF NOT EXISTS posts (
    //         id INTEGER PRIMARY KEY AUTOINCREMENT,
    //         user_id INTEGER NOT NULL,
    //         title TEXT NOT NULL,
    //         post_body TEXT NOT NULL,
    //         created_time TEXT NOT NULL
    //     )
    //     "#,
    //     [],
    // )?;

    conn.execute(
        "INSERT INTO posts (user_id, title, post_body, created_time) VALUES (1, ?1, ?2, datetime('now'))",
        (&post_title, &post_body),
    )?;

    Ok(())
}

#[server]
async fn get_posts() -> Result<Vec<Post>, ServerFnError> {
    let conn = rusqlite::Connection::open("./data.db3")?;

    let mut stmt =
        conn.prepare("SELECT id, title, post_body, created_time FROM posts ORDER BY id DESC")?;

    let post_iter = stmt.query_map([], |row| {
        Ok(Post {
            id: row.get(0)?,
            title: row.get(1)?,
            post_body: row.get(2)?,
            created_time: row.get(3)?,
        })
    })?;

    let mut posts = Vec::new();
    for post in post_iter {
        posts.push(post?);
    }

    Ok(posts)
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Post {
    id: u32,
    title: String,
    post_body: String,
    created_time: String,
}
