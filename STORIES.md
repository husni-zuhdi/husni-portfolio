# Stories for husni-portfolio repo

Inspired by bigboxSWE [video](https://www.youtube.com/watch?v=nqqmwRXSvrw) about finishing a (personal) programming project, I want to implement stories. The aim of this doc is to give us a well-defined and focused **scope**.

# Goals
1. To make a website to show my online presence.
2. To make a cool project.

## Work in Progress
    
## TODO
### User stories
- [ ] As an User, I want to experience quick loading when accessing husni zuhdi portfolio website.
    - We can improve our web speed by introducing in-memory caching on server-side.
    - Currently, `/blogs` took around 500ms to load while `/talks` took around 300ms.
    - `/blogs/BLOG_ID` took around 200-300ms to load depend on the images and another files.
    - I see some discussion in the Warframe Developer Discord channel about warframe-rs. They are using [moka](https://github.com/moka-rs/moka) for their API. I think we can explore this crate.
    - Step:
        1. Learn about [moka](https://github.com/moka-rs/moka) crate.
        2. Try to implement it (maybe on) `handler` module.
        3. If it works and can improve our website loading time. Let's be bulish lol.
        4. The first time to load might be not different, but the second-thrid-and-so-on should be faster. right?
- [ ] As an User, I need to zoom images to have a better view.
    - I can see images in mobile and I need to zoom it manually.
    - I think creating a zoom feature by clicking an image should do.
- [ ] As an User, I want to know the personality of the website creator.
    - Is it true? Does someone can know my personal based on my website?
    - As a website creator, I just want to make something cool. I don't care what others think of me.
    - I just want to make something that worth my time and effort!

### Admin stories

### Engineering stories
- [ ] As an Engineer, I want to have a nice CHANGELOG.md file to track my changes and versioning.
    - We can use [git-cliff](https://git-cliff.org/) but we need to upgrade the rust version to 1.8.3 or newer

## 0.3.2 2025-10-03
### User stories
- [x] As an User, I want to see whole code snippet without it's breaking the website in mobile.
    - This issue appear when you want to read blogs in mobile. The code snippet can be longer than the mobile screen widht.
    - What I think we can do are:
        1. Dynamically change the font size
        2. Add a limit on how long paragraph can go.
        3. In a code snippet, make it so we can horizontally scroll the snippet.
    - Applied the 2 and 3 suggestions by implementing `overflow: auto` on `<pre>` type.
    - We also encounter similar issue with `<table>` type but with different approach.
    - Table need to be wraped inside a div to make it responsive [ref](https://www.w3schools.com/howto/howto_css_table_responsive.asp)
    - However we can't alter the markdown rendering to wrap table inside a div.
    - So the current workaround is to add a notes in the add and edit blog pages.
    - To add a div manually in the markdown file :") Yeah it's not ideal but it's what we have.
    - Also we enabled allo dangerous html in the markdown compiler config.
### Admin stories
- [x] As an Admin, I want to have an administrator pages to manage my contents.
    - We have implemented the `Blogs` and `Talks` administrator pages.
    - We have implemented the `Tags` administrator pages.
    - Updated UI and core components
    - Admin pages are created.
- [x] As an Admin, I want to have a safe way to access my admin pages.
    - I think we can use authentication like `google` that match our google account only.
    - It's the frist time we play with authentication service. So excited! lol
    - For the initial step, we are implemented password-bassed auth with JWT
    - We can try to implement OAuth 2.0 next time

## 0.3.1 2025-08-15
### User stories
- [x] [As an User, I want to see another blog tags when I already clicked a tag](https://github.com/husni-zuhdi/husni-portfolio/pull/31)
    - Reffer to above story step 5.
    - Fixed the find_blogs query for turso database by separating query into two steps.
    - First is to find blog id with tags.
    - Second is to get tags and blogs from those blog ids.

### Admin stories
- [x] As an Admin, I want to have an access to edit blogs.
    - We can explore it by creating `/admin` page and working on how to edit a `talks` (since it's easiest than `blogs`).
    - Then we can work on the `blogs` feature.
    - At 2025-07-30 I start to work on Talk Admin page. We finally use HTMX in this project.
    - So far we tackle the edit button. Next we need to setup PUT endpoint.
    - There is an opportunity to improve the codebase readability.
    - We've finished the `talks` and `blogs` admin implementation.

### Engineering stories
- [x] As an Engineer, I want to finish my tech debt to properly implement `tags` and `blog_tag_mapping` tables on `blogs` databse adapter
    - As the title said, fix it please when you have time.
    - We need to fix this during/before we build our admin pages.
    - The tech debt is kind of paid with the current admin pages implementation.
    - We separate `tags` and `blog_tag_mappings` database implementation from `blogs`.
    - We did that to achieve segragation for each tables.


## 0.2.2 2025-04-29
### User stories
 - [x] As an User, I want to access husni zuhdi talk list
	 - User can access https://husni-zuhdi.com/talks to access husni zuhdi talk list
	 - Alternative, when user access https://husni-zuhdi.com they can click `Talks` on *Header* and *Bottom* to access husni zuhdi talk list
	 - Step:
		 1. Add a new route to `/talks`
		 2. Get the talk list from the database or API
            - It's harder than I thought. First, our code structure is not easy to modify if we want to add another tables.
            - What we need to do first is to add **functionality** first by preparing a new `talk` table.
            - Then we can work on a new database adapters for `memory` and `turso`
                - [x] Turso
                - [ ] `Memory`. There is no point to implement this I think. We'll skip this feature for now.
            - Then after all functionality okay, we can build our code to be more easy to modify by separating `blogs` and `talks` database adapters (?).
                - Database adapter is ready to be tested. Next we will implement the http handler and frontend.
         3. Present the `TALK_ID`, `TALK_NAME`, and if available `TALK_MEDIA_LINK` and `TALK_ORG_LINK`
- [x] As an User, I want to access each husni zuhdi talk record/video
	 - When user in https://husni-zuhdi.com/talks, they can click a `Play` hyperlink button to be redirected to the talk record/video
	 - The talk record/video will be opened in a new tab
	 - Step:
		 1. In the talk list page, User can access each talk record/video (if available) by clicking the `TALK_LINK`
- [x] As an User, I want to filter blogs based on tags
    - I can implement it by adding tags and update the `get_blogs` function with tags filter.
    - Steps:
        1. Add `BlogTags` on `Blog` model.
            - I think... it's counterproductive to update all `port`, `repo`, and `usecase` everytime we update the `model`.
            - It's time to improve our code base. I propose to update the code architecture on `add` and `update`.
            - `add` and `update` will receive `Blog` struct instead of individual fileds.
            - `Blog` struct should take Option fileds to accomodate partial fields `update`.
            - Except for Blog id (since we need it for all of the operation).
        2. Implement new Blog model on the database adapter.
            - We found an issue to implement tags in turso.
            - The query is become a bit complicated and I need to find a way to address this.
            - Initally, I want to use something simple like `LIKE` statement to filter tags in the `blogs` table.
            - There is a common pitfall with `LIKE` statment. Example: if I want to find blog with tag `cloud` only, it will return blog with tags `cloud` and `cloud-run`.
            - From this stackoverflow article, I think we can implement blog_tag_mapping on the database [link](http://stackoverflow.com/questions/51128832/what-is-the-best-way-to-design-a-tag-based-data-table-with-sqlite).
            - I will try to implement those in our database.
            - Another benefit is we don't need to update the current schema but the `find_blogs` query become more complex.
            - Okay it's done. I found out that we don't need a lifetime to render askama templates.
            - I'll delete all of the lifetime since they made my head explode (smoothbrain problem).
        3. Update the database schema.
        4. Update the `get_blog` handler and frontend to show tags.
        5. Update the `get_blogs` handler to add filter function based on URL parameter.
            - We finished the implementation of tag fe but found an issue on UX.
            - [ ] If we click a tag, the other tags in from a same blog disepear.
                - Still thinking about it
                - I don't think we need to fix it immediately. We'll track it in another story
            - [x] If we click a tag, the current active tag cannot be clicked to revert the condition.
                - In the `BlogsTemplate` I can add a field to track `active_tags`.
                - In the html template, loop the tags and check if it in the `active_tags`.
                - If the tag is in the `active_tags` list:
                    - Change the button color
                    - Override anchor to point to `/blogs` without any parameters
### Engineering stories
 - [x] As an Engineer, I want to implement compression middleware
	 - We can use `tower_http` as middleware to enable gzip compression [1](https://docs.rs/tower-http/0.6.2/tower_http/compression/index.html)
	 - Step:
		 1. Enable `compression-gzip` feature on `tower_http` rust cargo
		 2. Add a compression layer on axum router
 - [x] As an Engineer, I want to implement sqlite database
	 - We can use `sqlx` with `sqlite`  feature to implement sqlite database
	 - However, using `sqlite` and `turso` cause a cc linking issue due to these multiple c program have same functions
	 - So I think we need to stick to `turso` since we can still use `sqlite` with turso
	 - Step:
		 1. Implement `sqlite` database with `sqlx`
		 2. Test the implementation
		 3. (During turso implementation) Migrate `sqlx` to `turso` to solve cc linking issue
 - [x] As an Engineer, I want to implement turso database
	 - Register to https://turso.tech/ to get a free database
	 - Build dev and prod databases
	 - Implement `turso` and `sqlite`
	 - Step:
		 1. Learn how to use turso [1](https://docs.turso.tech/introduction)
		 2. Implement turso database
 - [x] As an Engineer, I want to fix the blogs turso implementation.
    - During the `talks` page revamp, we find a way to improve our code structure.
    - I want to bring what I find to `blogs` feature.
    - It simplify hexagonal arch structure and provide better way to extend features.
    - Step:
        1. Update `blogs` implementation to match with the `talks`.
        2. Fix any bug/error during the update process.
- [x] As an Engineer, I want to migrate my blogs data from github to fully database(s).
    - This might introduce a BREAKING CHANGES since we won't support Github blogs pulling.
    - We already move our current blogs data to turso dev database.
    - Let's move it to the release database.
    - Step:
        1. Populate `release` turso database with the latest data.
        2. Remove (or disable) the github api feature.
        3. Fully use the database in the future.

## 0.2.1 and below
### User Stories
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
 - [x] As an User, I want to see a decent looking website
	 - Implement simple styling with tailwind css
	 - We are inspired with [Carson Gross](https://bigsky.software/cv/) CV website
	 - Step:
		 1. Setup an initial tailwindcss config to be use on this project
		 2. Generate one css file to be used across all pages (for simplicity)

### Engineering Stories
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
