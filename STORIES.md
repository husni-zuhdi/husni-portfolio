# Stories for husni-portfolio repo

Inspired by bigboxSWE [video](https://www.youtube.com/watch?v=nqqmwRXSvrw) about finishing a (personal) programming project, I want to implement stories. The aim of this doc is to give us a well-defined and focused **scope**.

## User Stories
 - [x] As an User, I want to access husni zuhdi portfolio
	 - User can access https://husni-zuhdi.com to access husni zuhdi portfolio
	 - Step:
		 1. Start a rust project to render HTML templates
		 2. Fill the home/portfolio page with designated information such us description, experience, etc
		 3. Build a docker container for the rust project
		 4. Deploy the container in the Cloud
 - [x] As an User, I want to access husni zuhdi blog list
	 - User can access https://husni-zuhdi.com/blogs to access husni zuhdi blog list
	 - Alternative, when user access https://husni-zuhdi.com they can click `Blogs` on *Header* and *Bottom* to access husni zuhdi blog list
	 - Step:
		 1. Add a new route to `/blogs`
		 2. Get the blog list from the database or another API
		 3. Present the `BLOG_ID` and `BLOG_NAME`
 - [x] As an User, I want to access husni zuhdi blog reading
	 - User can access https://husni-zuhdi.com/blogs/BLOG_ID to access husni zuhdi blog with blog Id = `BLOG_ID`
	 - Alternative, when user access https://husni-zuhdi.com/blogs they can click the `BLOG_NAME` hyperlink to access husni zuhdi blog with name `BLOG_NAME`
	 - Step:
		 1. Add new routes for each `BLOG_ID` similar to  `/blogs/BLOG_ID`
		 2. When user directed to blog, render the `BLOG_NAME` and `BLOG_BODY` from the database
 - [x] As an User, I want to see not found page when I access a not available pages
	 - When user access not available pages, they will be presented with 404 not found page
	 - Step:
		 1. Set a `fallback` in the axum router to show 404 page
 - [ ] As an User, I want to access husni zuhdi talk list
	 - User can access https://husni-zuhdi.com/talks to access husni zuhdi talk list
	 - Alternative, when user access https://husni-zuhdi.com they can click `Talks` on *Header* and *Bottom* to access husni zuhdi talk list
	 - Step:
		 1. Add a new route to `/talks`
		 2. Get the talk list from the database or API
		 3. Present the `TALK_ID`, `TALK_NAME`, and if available `TALK_LINK`
 - [x] As an User, I want to access each husni zuhdi talk record/video
	 - When user in https://husni-zuhdi.com/talks, they can click a `Play` hyperlink button to be redirected to the talk record/video
	 - The talk record/video will be opened in a new tab
	 - Step:
		 1. In the talk list page, User can access each talk record/video (if available) by clicking the `TALK_LINK`
 - [x] As an User, I want to see a decent looking website
	 - Implement simple styling with tailwind css
	 - We are inspired with [Carson Gross](https://bigsky.software/cv/) CV website
	 - Step:
		 1. Setup an initial tailwindcss config to be use on this project
		 2. Generate one css file to be used across all pages (for simplicity)

## Engineering Stories
 - [x] As an Engineer, I want to implement HTML templating
	 - We use `askama` for HTML templating on rust
	 - For the API server, we use `axum`
	 - Step:
		 1. Learn how to use `axum` and `askama`
		 2. Implement it on this project
 - [x] As an Engineer, I want to push husni zuhdi portfolio docker image to Artifact Registry
	 - Push a test docker image with `gcloud` command
	 - Then setup a github action to push docker image to Artifact Registry automatically when a new tag available
	 - Step:
		 1. Push the test docker image to Artifact Registry with `gcloud` command
		 2. Create a simple push image github action to push a new image when we create a new tag
 - [x] As an Engineer, I want to deploy husni zuhdi portfolio in Cloud Run
	 - Test it first by deploying manually with `gcloud` cli then we can build a terrafrom workflow
	 - Step:
		 1. Deploy the husni zuhdi portfolio container with `gcloud` cli
		 2. Deploy the husni zuhdi portfolio container with terraform workflow
 - [x] As an Engineer, I want to implement compression middleware
	 - We can use `tower_http` as middleware to enable gzip compression [1](https://docs.rs/tower-http/0.6.2/tower_http/compression/index.html)
	 - Step:
		 1. Enable `compression-gzip` feature on `tower_http` rust cargo
		 2. Add a compression layer on axum router
 - [ ] As an Engineer, I want to implement sqlite database
	 - We can use `sqlx` with `sqlite`  feature to implement sqlite database
	 - However, using `sqlite` and `turso` cause a cc linking issue due to these multiple c program have same functions
	 - So I think we need to stick to `turso` since we can still use `sqlite` with turso
	 - Step:
		 1. Implement `sqlite` database with `sqlx`
		 2. Test the implementation
		 3. (During turso implementation) Migrate `sqlx` to `turso` to solve cc linking issue
 - [ ] As an Engineer, I want to implement turso database
	 - Register to https://turso.tech/ to get a free database
	 - Build dev and prod databases
	 - Implement `turso` and `sqlite`
	 - Step:
		 1. Learn how to use turso [1](https://docs.turso.tech/introduction)
		 2. Implement turso database
