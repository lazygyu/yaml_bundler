# What is this?

This is a tool to bundle my yaml files. I want to write an OAS3 document but it feels hard tome to keep the contents in a single file. So I added some simple tweaks like `$include` and `$generic` for my purpose.

# How to use?

This program parse the yaml file given and then process some additional keywords like below;

## $include

Including files in the path provided. The path can be a glob pattern.

### Syntax

```yaml
$include: {pattern}
```

The `pattern` points a path or a glob pattern for targets to include.

### Usage

#### Files
```yaml
# ---- file: main.yaml ----
anObject:
	$include: ./objects/**/*.yaml

# ---- file: objects/obj1.yaml ----
obj1:
  prop1: value1
	prop2:
		- item1
		- item2

# ---- file: objects/obj2.yaml ----
obj2:
	prop1: value2
	prop2:
		type: object
		properties:
			some: thing
			is: going on
```

#### Running the command
```bash
> apidoc-builder ./main.yaml
```

#### Result

```yaml
anObject:
	obj1:
		prop1: value1
		prop2:
			- item1
			- item2
	obj2:
		prop1: value2
		prop2:
			type: object
			properties:
				some: thing
				is: going on
```

## $generic

The name `generic` might be not fit to this feature but I didn't want to struggle with the name.

You can define a generic with a key name which ends with `<GENERIC>`. The definition will be removed in the result YAML file.

### Syntax


### Usage

The source is like;

```yaml
# --- definition part ---
DefaultResponse<GENERIC>:		# this defines a generic with name DefaultResponse
	description: success
	content:
		application/json:
			schema:
				type: object
				properties:
					data: DATA_TYPE    # DATA_TYPE will be replaced by something provided
					error: boolean

# --- using part ---
responses:
	200:
		$generic:
			target: DefaultResponse
			DATA_TYPE:								# You can provide the replacement for DATE_TYPE like this
				type: array
				items:
					type: number
```

And the built result will be:

```yaml
responses:
	200:
		description: success
		content:
			application/json:
				schema:
					type: object
					properties:
						data:
							type: array
							items:
								type: number
					error: boolean
							
```
