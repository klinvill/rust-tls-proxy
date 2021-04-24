import json

# each value in input is an array even if there is only one element
def jsonify_urllib_params(input_dict):
	new_dict = {}
	for key in input_dict:
		# ignore metadeta (key starting with _)
		if key[0] != "_":
			val = input_dict[key]
			if len(val) == 1:
				val = val[0]
			new_dict[key] = val
	return json.dumps(new_dict)

def unescape(in_str):
	new_str = ""
	length = len(in_str)
	i = 0
	while i < length:
		c = in_str[i]
		if c == "\\" and i+1 < length:
			new_str += in_str[i+1]
			i += 1
		elif c != "\\":
			new_str += c
		i += 1
	return new_str


# I did this in case any messages have newline characters (so I won't necessarily store comments as one line the posts file)
# Also, the json.dumps inserts escape characters before special characters (to tell textual " apart from json ") so I "unescape" them when producing the dictionary values.
def read_simple_json(json_str, start_idx):
	return_dict = {}
	in_quotes = False
	in_esc = False
	quote_start = None
	param = None
	seeker = start_idx
	offset = start_idx-1
	valid_json = False
	length = len(json_str) # length of whole json string, not substring after start index

	while not valid_json:
		# in case we need to repeat the outer loop and look for a valid json at different "{"
		seeker = offset+1
		while (seeker < length and json_str[seeker] != "{"):
			seeker += 1
		if(seeker == length):
			return None
		else:
			offset = seeker # index of "{" starting the json

		# try to parse the json string
		while seeker < length:
			cur_char = json_str[seeker]
			if not in_quotes and not in_esc and cur_char == "}":
				valid_json = True
				break
			elif in_esc:
				in_esc = False
			elif cur_char == "\\":
				in_esc = True
			# handle quotes
			elif cur_char == "\"":
				if not in_quotes:
					quote_start = seeker
				if in_quotes:
					if param == None and seeker+1 < length and json_str[seeker+1] == ":":
						param = unescape(json_str[quote_start+1:seeker])
					else:
						return_dict[param] = unescape(json_str[quote_start+1:seeker])
						param = None
					quote_start = None
				in_quotes = not in_quotes
			seeker += 1
		# end of while loop
	return_dict['_offset'] = offset - start_idx # how many chars to seek to the start of the json
	return_dict['_length'] = seeker+1 - offset # length of the json in chars
	return return_dict
