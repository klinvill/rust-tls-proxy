import json

# each value in input is an array even if there is only one element
def jsonify_urllib_params(input_dict):
	new_dict = {}
	for key in input_dict:
		# filter quote from key and ignore metadeta (key starting with _)
		nk = filter_out_char(key, "\"")
		if key[0] != "_":
			val = input_dict[key]
			if len(val) == 1:
				val = val[0]
			new_dict[nk] = val
	return json.dumps(new_dict)

# I could use the built-in filter function, but this is a little more clear
def filter_out_char(in_str, rem_ch):
	new_str = ""
	for c in in_str:
		if c != rem_ch:
			new_str += c
	return new_str

# I did this in case any messages have newline characters (so I won't necessarily store comments as one line the posts file)
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
						param = json_str[quote_start+1:seeker]
					else:
						return_dict[param] = json_str[quote_start+1:seeker]
						param = None
					quote_start = None
				in_quotes = not in_quotes
			seeker += 1
		# end of while loop
	return_dict['_offset'] = offset - start_idx # how many chars to seek to the start of the json
	return_dict['_length'] = seeker+1 - offset # length of the json in chars
	return return_dict