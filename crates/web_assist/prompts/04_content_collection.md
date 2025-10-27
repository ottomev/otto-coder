# Stage 4: Content Collection & SEO

## Duration: 6 hours

## Project Context
{{project_context}}

## Your Mission

Create ALL website content optimized for SEO and ready for implementation. Client **APPROVAL** required before development.

## Content Requirements

### 1. Homepage Copy
- Hero section headline and subheadline
- Value proposition
- Key features/benefits (3-5 sections)
- Call-to-action text
- Trust signals (testimonials, stats, logos)

### 2. Page Content
Write content for:
- About page
- Services/Products pages
- Contact page
- Any additional pages

### 3. SEO Optimization
For each page:
- Meta title (50-60 characters)
- Meta description (150-160 characters)
- H1 heading
- H2-H6 subheadings
- Primary keywords
- Secondary keywords
- Internal linking strategy

### 4. Imagery & Media
- Image descriptions for needed photos
- Alt text for all images
- Icon requirements
- Video/animation needs (if any)

## Deliverables

Create in `deliverables/04_content/`:

### 1. Page Content Files
- `homepage.md` - Complete homepage copy
- `about.md` - About page content
- `services.md` - Services/products content
- `contact.md` - Contact page content

### 2. seo_meta.json
```json
{
  "pages": {
    "homepage": {
      "title": "...",
      "description": "...",
      "keywords": ["keyword1", "keyword2"],
      "og_image": "path/to/image"
    },
    "about": { ... },
    "services": { ... }
  },
  "site": {
    "title": "Company Name",
    "tagline": "...",
    "description": "..."
  }
}
```

### 3. imagery_requirements.md
List all images needed:
```markdown
# Homepage
- Hero image: [description, dimensions]
- Feature 1 image: [description]
- etc.

# About Page
- Team photo: [description]
- etc.
```

### 4. style_guide.md
Content guidelines:
- Tone of voice
- Brand personality
- Writing style
- Key messaging
- Words to use/avoid

## Success Criteria

- [ ] All page content written
- [ ] SEO optimized (titles, descriptions, keywords)
- [ ] Professional, engaging tone
- [ ] Free of errors
- [ ] Aligned with target audience
- [ ] Ready for client approval

Begin content creation now!
