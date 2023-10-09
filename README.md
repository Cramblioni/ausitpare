# ausiatpare
## auto site parser & renderer

A simple Apricot style renderer, in various languages.

ausiatpare doesn't have a complicated interface. STDERR is used for logging and
STDOUT is used for progress updates. progress updates use a format that is
easily parsable, which should help when using this in other applications.

## expected USAGE message
```
USAGE: ausiatpare [templates] [pages]
	[templates] - Path to a folder containing the templates
	[pages] - path to a folder containing the pages
```

## some grammar stuff
```sigrala
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

### future ideas
- caching
	- cache compiled code
	- cache expansions per scope
