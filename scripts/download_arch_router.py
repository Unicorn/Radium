#!/usr/bin/env python3
"""
Download and cache the Arch-Router 1.5B model from HuggingFace.

This script downloads the model and tests basic routing functionality.
"""

import json
import os
import sys
from pathlib import Path

def download_model():
    """Download Arch-Router model from HuggingFace."""
    try:
        from transformers import AutoModelForCausalLM, AutoTokenizer
        print("✓ transformers library found")
    except ImportError:
        print("✗ transformers library not found")
        print("Installing transformers...")
        import subprocess
        subprocess.check_call([sys.executable, "-m", "pip", "install", "transformers>=4.37.0", "--quiet"])
        from transformers import AutoModelForCausalLM, AutoTokenizer
        print("✓ transformers installed")

    model_name = "katanemo/Arch-Router-1.5B"
    cache_dir = Path(__file__).parent.parent / "models" / "arch-router"
    cache_dir.mkdir(parents=True, exist_ok=True)

    print(f"\nDownloading {model_name}...")
    print(f"Cache directory: {cache_dir}")
    print("This may take several minutes (model is ~2GB)...\n")

    try:
        # Download model
        model = AutoModelForCausalLM.from_pretrained(
            model_name,
            device_map="auto",
            torch_dtype="auto",
            trust_remote_code=True,
            cache_dir=str(cache_dir)
        )
        print("✓ Model downloaded successfully")

        # Download tokenizer
        tokenizer = AutoTokenizer.from_pretrained(
            model_name,
            cache_dir=str(cache_dir)
        )
        print("✓ Tokenizer downloaded successfully")

        return model, tokenizer, cache_dir

    except Exception as e:
        print(f"✗ Error downloading model: {e}")
        return None, None, None


def test_model(model, tokenizer):
    """Test basic routing functionality."""
    print("\n--- Testing Model Inference ---\n")

    TASK_INSTRUCTION = """
You are a helpful assistant designed to find the best suited route.
You are provided with route description within <routes></routes> XML tags:
<routes>

{routes}

</routes>

<conversation>

{conversation}

</conversation>
"""

    FORMAT_PROMPT = """
Your task is to decide which route is best suit with user intent on the conversation in <conversation></conversation> XML tags. Follow the instruction:
1. If the latest intent from user is irrelevant or user intent is full filled, response with other route {"route": "other"}.
2. You must analyze the route descriptions and find the best match route for user latest intent.
3. You only response the name of the route that best matches the user's request, use the exact name in the <routes></routes>.

Based on your analysis, provide your response in the following JSON formats if you decide to match any route:
{"route": "route_name"}
"""

    def format_prompt(route_config, conversation):
        return (
            TASK_INSTRUCTION.format(
                routes=json.dumps(route_config),
                conversation=json.dumps(conversation)
            )
            + FORMAT_PROMPT
        )

    # Test case 1: Code generation
    print("Test 1: Code generation task")
    route_config = [
        {"name": "code_generation", "description": "Generating new code, functions, or boilerplate"},
        {"name": "code_review", "description": "Reviewing code for quality, security, and best practices"},
        {"name": "bug_fixing", "description": "Identifying and fixing errors or bugs"},
    ]

    conversation = [{"role": "user", "content": "Write a Python function to calculate fibonacci numbers"}]

    route_prompt = format_prompt(route_config, conversation)
    messages = [{"role": "user", "content": route_prompt}]

    input_ids = tokenizer.apply_chat_template(
        messages, add_generation_prompt=True, return_tensors="pt"
    ).to(model.device)

    generated_ids = model.generate(input_ids=input_ids, max_new_tokens=100)
    prompt_lengths = input_ids.shape[1]
    generated_only = [output_ids[prompt_lengths:] for output_ids in generated_ids]

    response = tokenizer.batch_decode(generated_only, skip_special_tokens=True)[0]
    print(f"Input: {conversation[0]['content']}")
    print(f"Output: {response}")

    try:
        parsed = json.loads(response)
        print(f"✓ Parsed route: {parsed.get('route')}\n")
    except:
        print(f"✗ Failed to parse JSON response\n")

    # Test case 2: Bug fixing
    print("Test 2: Bug fixing task")
    conversation = [{"role": "user", "content": "Fix this error: TypeError: 'NoneType' object is not subscriptable"}]

    route_prompt = format_prompt(route_config, conversation)
    messages = [{"role": "user", "content": route_prompt}]

    input_ids = tokenizer.apply_chat_template(
        messages, add_generation_prompt=True, return_tensors="pt"
    ).to(model.device)

    generated_ids = model.generate(input_ids=input_ids, max_new_tokens=100)
    prompt_lengths = input_ids.shape[1]
    generated_only = [output_ids[prompt_lengths:] for output_ids in generated_ids]

    response = tokenizer.batch_decode(generated_only, skip_special_tokens=True)[0]
    print(f"Input: {conversation[0]['content']}")
    print(f"Output: {response}")

    try:
        parsed = json.loads(response)
        print(f"✓ Parsed route: {parsed.get('route')}\n")
    except:
        print(f"✗ Failed to parse JSON response\n")


def main():
    """Main entry point."""
    print("=" * 60)
    print("Arch-Router 1.5B Model Download & Test")
    print("=" * 60)

    model, tokenizer, cache_dir = download_model()

    if model is None:
        print("\n✗ Model download failed. Exiting.")
        return 1

    print(f"\n✓ Model cached at: {cache_dir}")

    # Test the model
    test_model(model, tokenizer)

    print("\n" + "=" * 60)
    print("✓ Download and test complete!")
    print("=" * 60)

    return 0


if __name__ == "__main__":
    sys.exit(main())
