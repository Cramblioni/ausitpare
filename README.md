# ausitpare
## auto site parser & renderer

A simple Apricot style renderer, in various languages.

ausitpare doesn't have a complicated interface. STDERR is used for logging and
STDOUT is used for progress updates. progress updates use a format that is
easily parsable, which should help when using this in other applications.

## expected USAGE message
```
USAGE: ausitpare [templates] [pages]
	[templates] - Path to a folder containing the templates
	[pages] - path to a folder containing the pages
```

### further notes
- an attribute must be closed
	- leave un-substituted
- conditionals must be closed
	- I don't know
- All Apricot Elements must not appear in the output file


## Semantics

Where Apricot uses these elements as various control things, ausiatpare treats
them as instructions. This makes using them in ausiatpare more complicated, but
makes the elements more flexible. We treat both templates and pages as source
code, with the page being compiled, `template` being looked up, and then the
correct template being compiled and executed.

For example, the following source:
```ausitpare
<!-- attrib msg : Hello, World! -->
<p> all I got to say is "[#msg#]", thank you! </p>
```
becomes the following instructions:
```
msg:
	put_text "Hello, World!"
	proceed
_content:
	put_text "<p> all I got to say is "
	invoke msg
	put_text ", thank you! </p>"
	proceed
```
each attribute is scoped to the current attribute, with dynamic lookup. Each
file has the implicit `_content` attribute, which is visible to templates as
`content`. This natural nesting is quite useful. Any failed name lookup will
result in a warning message (or empty string). Because accessing a attribute
invokes it, conditionals do execute their parameters. Conditionals use simple
unification to compare the strings. 
# Todo Lists
## required
- [ ] Add new compiler and evaluation system
    - [ ] Add new evaluator
    - [ ] Add new compiler
- [ ] Special Attributes
    - [ ] root (i need to look into how Apricot does it)
    - [ ] modified (copy Apricot formatting)
    - [ ] path
    - [ ] content
- [ ] Add scoping (It mustn't break compatibility with Apricot) 
- [ ] Disallow empty attribute name `[##]`
- [ ] make syntactic elements more whitespace agnostic
    (so `<!--attribute dave:test-->` is allowable)
- [ ] Add page nesting (planned a `[## ... ##]` syntax)
    The syntax is to have a ausitpare compile a new page and hook it up to be
    used by the rest of the code. **CURRENTLY** It's planned to have the
    `content` attribute of the page to be immediately invoked. The syntax is to
    allow for selecting a page and pre-setting attributes.
    - [ ] Add custom parse rules
    - [ ] Stricten Scoping

## future ideas
- caching
	- cache compiled code
	- cache expansions per scope
- nesting
    - pseudo-namespacing
        so `[## magic ##]` would allow `[#magic:path#]` to be used.
