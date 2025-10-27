# Stage 6: Quality Assurance & Testing

## Duration: 4 hours

## Project Context
{{project_context}}

## Your Mission

Test the website thoroughly and ensure it's production-ready.

## Testing Areas

### 1. Functionality Testing (1 hour)
- [ ] All links work correctly
- [ ] Navigation works on all pages
- [ ] Contact form submits successfully
- [ ] Form validation works
- [ ] All interactive elements function
- [ ] No JavaScript errors in console

### 2. Cross-Browser Testing (1 hour)
Test in:
- [ ] Chrome
- [ ] Firefox
- [ ] Safari
- [ ] Edge

Verify:
- Consistent appearance
- All features work
- No layout issues
- No browser-specific bugs

### 3. Cross-Device Testing (1 hour)
Test on:
- [ ] Desktop (1920px, 1440px)
- [ ] Tablet (768px, iPad)
- [ ] Mobile (375px, iPhone)
- [ ] Large mobile (414px, iPhone Plus)

Verify:
- Responsive layout
- Touch interactions work
- Text is readable
- Images scale properly
- No horizontal scroll

### 4. Performance & Optimization (30 minutes)
Run tests and optimize:
- [ ] Lighthouse score > 90
- [ ] Core Web Vitals pass
- [ ] Page load time < 3s
- [ ] Images optimized
- [ ] No render-blocking resources

### 5. Accessibility Testing (30 minutes)
- [ ] Screen reader compatible
- [ ] Keyboard navigation works
- [ ] ARIA labels present
- [ ] Color contrast meets WCAG AA
- [ ] Focus indicators visible
- [ ] Alt text on all images

## Bug Fixes

Document and fix ALL bugs found:
- Create list in `deliverables/06_qa/bugs_found.md`
- Fix each bug
- Re-test after fixes
- Mark as resolved

## Deliverables

Create in `deliverables/06_qa/`:

### 1. test_report.md
```markdown
# QA Test Report

## Functionality Tests
- Navigation: ✅ Pass
- Forms: ✅ Pass
- Links: ✅ Pass
[etc.]

## Cross-Browser Results
- Chrome: ✅ Pass
- Firefox: ✅ Pass
- Safari: ✅ Pass
- Edge: ✅ Pass

## Cross-Device Results
- Desktop: ✅ Pass
- Tablet: ✅ Pass
- Mobile: ✅ Pass

## Issues Found
[List any issues and their status]

## Overall Status
Ready for client preview: YES/NO
```

### 2. performance_report.md
```markdown
# Performance Report

## Lighthouse Scores
- Performance: 95
- Accessibility: 100
- Best Practices: 100
- SEO: 100

## Core Web Vitals
- LCP: 1.2s ✅
- FID: 12ms ✅
- CLS: 0.02 ✅

## Optimizations Made
- [List optimizations]
```

### 3. bugs_found.md (if any)
Document all bugs found and fixed

## Success Criteria

- [ ] All tests pass
- [ ] No critical bugs
- [ ] Performance excellent
- [ ] Accessibility compliant
- [ ] Works on all browsers
- [ ] Works on all devices
- [ ] Ready for client preview

Begin QA testing now!
